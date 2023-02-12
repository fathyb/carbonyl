use core::mem::MaybeUninit;
use std::{io, str::FromStr};

use crate::{cli::CommandLine, gfx::Size, utils::log};

#[derive(Clone, Debug)]
pub struct Window {
    pub dpi: f32,
    /// Size of a terminal cell in pixels
    pub scale: Size<f32>,
    /// Size of the termina window in cells
    pub cells: Size,
    /// Size of the browser window in pixels
    pub browser: Size,
    pub cmd: CommandLine,
}

impl Window {
    pub fn read() -> io::Result<Window> {
        let mut window = Self {
            dpi: 1.0,
            scale: (0.0, 0.0).into(),
            cells: (0, 0).into(),
            browser: (0, 0).into(),
            cmd: CommandLine::parse(),
        };

        window.update()?;

        Ok(window)
    }

    pub fn update(&mut self) -> io::Result<&Self> {
        let mut ptr = MaybeUninit::<libc::winsize>::uninit();
        let mut size = unsafe {
            if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ptr.as_mut_ptr()) == 0 {
                Some(ptr.assume_init())
            } else {
                None
            }
        }
        .ok_or_else(io::Error::last_os_error)?;

        if size.ws_col == 0 || size.ws_row == 0 {
            let cols = parse_var("COLUMNS").unwrap_or(80);
            let rows = parse_var("LINES").unwrap_or(24);

            log::warning!(
                "TIOCGWINSZ returned an empty size ({}x{}), defaulting to {}x{}",
                size.ws_col,
                size.ws_row,
                cols,
                rows
            );

            size.ws_col = cols;
            size.ws_row = rows;
        }

        let zoom = 1.5 * self.cmd.zoom;
        let cells = Size::new(size.ws_col.max(1), size.ws_row.max(2) - 1);
        let auto_scale = false;
        let cell_pixels = if auto_scale {
            Size::new(size.ws_xpixel as f32, size.ws_ypixel as f32) / cells.cast()
        } else {
            Size::new(8.0, 16.0)
        };
        // Normalize the cells dimensions for an aspect ratio of 1:2
        let cell_width = (cell_pixels.width + cell_pixels.height / 2.0) / 2.0;

        self.dpi = 2.0 / cell_width * zoom;
        // A virtual cell should contain a 2x4 pixel quadrant
        self.scale = Size::new(2.0, 4.0) / self.dpi;
        // Keep some space for the UI
        self.cells = Size::new(size.ws_col.max(1), size.ws_row.max(2) - 1).cast();
        self.browser = self.cells.cast::<f32>().mul(self.scale).ceil().cast();

        Ok(self)
    }
}

fn parse_var<T: FromStr>(var: &str) -> Option<T> {
    std::env::var(var).ok()?.parse().ok()
}
