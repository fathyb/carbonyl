use core::mem::MaybeUninit;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    os::fd::AsRawFd,
    str::FromStr,
    time::{Duration, Instant},
};

use crate::{cli::CommandLine, gfx::Size, utils::log};

/// A terminal window.
#[derive(Clone, Debug)]
pub struct Window {
    /// Device pixel ratio
    pub dpi: f32,
    /// Size of a terminal cell in pixels
    pub scale: Size<f32>,
    /// Size of the termina window in cells
    pub cells: Size,
    /// Size of the browser window in pixels
    pub browser: Size,
    /// Full terminal pixel geometry for graphics output
    pub graphics_px: Size,
    /// Device scale factor used by Chromium (integer preferred)
    pub dsf: f32,
    /// Command line arguments
    pub cmd: CommandLine,
}

impl Window {
    /// Read the window
    pub fn read() -> Window {
        let mut window = Self {
            dpi: 1.0,
            scale: (0.0, 0.0).into(),
            cells: (0, 0).into(),
            browser: (0, 0).into(),
            graphics_px: (0, 0).into(),
            dsf: 1.0,
            cmd: CommandLine::parse(),
        };

        window.update();

        window
    }

    pub fn update(&mut self) -> &Self {
        let (mut term, cell) = unsafe {
            let mut ptr = MaybeUninit::<libc::winsize>::uninit();

            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ptr.as_mut_ptr()) == 0 {
                let size = ptr.assume_init();

                (
                    Size::new(size.ws_col, size.ws_row),
                    Size::new(size.ws_xpixel, size.ws_ypixel),
                )
            } else {
                (Size::splat(0), Size::splat(0))
            }
        };

        if term.width == 0 || term.height == 0 {
            let cols = match parse_var("COLUMNS").unwrap_or(0) {
                0 => 80,
                x => x,
            };
            let rows = match parse_var("LINES").unwrap_or(0) {
                0 => 24,
                x => x,
            };

            log::warning!(
                "TIOCGWINSZ returned an empty size ({}x{}), defaulting to {}x{}",
                term.width,
                term.height,
                cols,
                rows
            );

            term.width = cols;
            term.height = rows;
        }

        let mut cell_pixels =
            if term.width > 0 && term.height > 0 && cell.width > 0 && cell.height > 0 {
                Size::new(
                    cell.width as f32 / term.width.max(1) as f32,
                    cell.height as f32 / term.height.max(1) as f32,
                )
            } else {
                Size::new(0.0, 0.0)
            };

        if cell_pixels.width <= 0.0 || cell_pixels.height <= 0.0 {
            if let Some(win_px) = query_window_pixels() {
                cell_pixels = Size::new(
                    win_px.width / term.width.max(1) as f32,
                    win_px.height / term.height.max(1) as f32,
                );
            }

            if cell_pixels.width <= 0.0 || cell_pixels.height <= 0.0 {
                cell_pixels = query_cell_geometry().unwrap_or(Size::new(8.0, 16.0));
            }
        }
        // Normalize the cells dimensions for an aspect ratio of 1:2
        self.scale = cell_pixels;
        // Keep some space for the UI
        self.cells = Size::new(term.width.max(1), term.height.max(2) - 1).cast();
        self.graphics_px = Size::new(
            (self.cells.width as f32 * cell_pixels.width).round() as u32,
            (self.cells.height as f32 * cell_pixels.height).round() as u32,
        );

        // Choose an integer device scale factor (DSF) to avoid fractional raster scaling.
        let mut dsf = if cell_pixels.height >= 18.0 { 2.0 } else { 1.0 };
        if let Ok(value) = std::env::var("CARBONYL_DSF") {
            match value.trim() {
                "1" => dsf = 1.0,
                "2" => dsf = 2.0,
                "3" => dsf = 3.0,
                _ => {}
            }
        }
        self.dsf = dsf;
        self.dpi = self.dsf;

        self.browser = Size::new(
            (self.graphics_px.width as f32 / self.dsf).round() as u32,
            (self.graphics_px.height as f32 / self.dsf).round() as u32,
        );

        self
    }
}

fn parse_var<T: FromStr>(var: &str) -> Option<T> {
    std::env::var(var).ok()?.parse().ok()
}

fn query_cell_geometry() -> Option<Size<f32>> {
    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .ok()?;
    let fd = tty.as_raw_fd();
    let mut term = MaybeUninit::<libc::termios>::uninit();

    unsafe {
        if libc::tcgetattr(fd, term.as_mut_ptr()) != 0 {
            return None;
        }
    }

    let original = unsafe { term.assume_init() };
    let mut raw = original;
    let c_oflag = raw.c_oflag;

    unsafe {
        libc::cfmakeraw(&mut raw);
    }

    raw.c_oflag = c_oflag;

    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
        return None;
    }

    struct Restore(libc::c_int, libc::termios);

    impl Drop for Restore {
        fn drop(&mut self) {
            unsafe {
                libc::tcsetattr(self.0, libc::TCSANOW, &self.1);
            }
        }
    }

    let _restore = Restore(fd, original);

    if tty.write_all(b"\x1b[16t").is_err() || tty.flush().is_err() {
        return None;
    }

    let mut buffer = [0u8; 128];
    let mut length = 0usize;
    let deadline = Instant::now() + Duration::from_millis(100);

    while length < buffer.len() && Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let timeout = remaining.as_millis().min(i32::MAX as u128) as libc::c_int;
        let mut fds = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };

        let result = unsafe { libc::poll(&mut fds, 1, timeout) };

        if result <= 0 {
            break;
        }

        match tty.read(&mut buffer[length..]) {
            Ok(0) => break,
            Ok(read) => {
                length += read;

                if buffer[..length].contains(&b't') {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    if length == 0 {
        return None;
    }

    let response = std::str::from_utf8(&buffer[..length]).ok()?;
    let start = response.rfind("\u{1b}[6;")?;
    let rest = &response[start + 3..];
    let end = rest.find('t')?;
    let mut parts = rest[..end].split(';');
    let height = parts.next()?.parse::<f32>().ok()?;
    let width = parts.next()?.parse::<f32>().ok()?;

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    Some(Size::new(width, height))
}

fn query_window_pixels() -> Option<Size<f32>> {
    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .ok()?;

    tty.write_all(b"\x1b[14t").ok()?;
    tty.flush().ok()?;

    let mut buf = [0u8; 128];
    let n = tty.read(&mut buf).ok()?;

    if n == 0 {
        return None;
    }

    let response = std::str::from_utf8(&buf[..n]).ok()?;
    let start = response.rfind("\u{1b}[4;")?;
    let rest = &response[start + 3..];
    let end = rest.find('t')?;
    let mut parts = rest[..end].split(';');
    let height = parts.next()?.parse::<f32>().ok()?;
    let width = parts.next()?.parse::<f32>().ok()?;

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    Some(Size::new(width, height))
}
