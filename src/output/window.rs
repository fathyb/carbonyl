use core::mem::MaybeUninit;
use std::str::FromStr;

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
            cmd: CommandLine::parse(),
        };

        window.update();

        window
    }

    pub fn update(&mut self) -> &Self {
        let (mut term, mut cell) = unsafe {
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

        // Whether the terminal reported real cell pixel dimensions (kitty does).
        let pixel_known = cell.width > 0 && cell.height > 0;

        if cell.width == 0 || cell.height == 0 {
            cell.width = 8;
            cell.height = 16;
        }

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

        let zoom = 1.5 * self.cmd.zoom;
        let cells = Size::new(term.width.max(1), term.height.max(2) - 1);
        let auto_scale = false;
        let cell_pixels = if auto_scale {
            Size::new(cell.width as f32, cell.height as f32) / cells.cast()
        } else {
            Size::new(8.0, 16.0)
        };
        // Normalize the cells dimensions for an aspect ratio of 1:2
        let cell_width = (cell_pixels.width + cell_pixels.height / 2.0) / 2.0;

        // Round DPI to 2 decimals for proper viewport computations
        let dpi = (2.0 / cell_width * zoom * 100.0).ceil() / 100.0;
        // In graphics mode the framebuffer is shown at full pixel resolution via
        // the kitty protocol, so render it at a higher device-pixel-ratio for a
        // crisp result. Scaling both the DPI and the per-cell pixel size by the
        // same factor keeps the page layout identical while quadrupling the
        // pixels behind each cell.
        let supersample = if self.cmd.graphics { 2.0 } else { 1.0 };
        // Device pixel ratio reported to Chromium
        self.dpi = dpi * supersample;
        // A virtual cell should contain a 2x4 pixel quadrant (times supersample)
        self.scale = Size::new(2.0, 4.0) / (dpi / supersample);

        // In graphics mode the framebuffer is scaled to fill the cell box. The
        // 2x4 quadrant assumes a 1:2 cell, but real terminal cells are usually
        // taller than that, which stretches the image vertically. Match the
        // framebuffer's aspect ratio to the reported cell so it isn't distorted.
        if self.cmd.graphics && pixel_known {
            let cell_w = cell.width as f32 / term.width.max(1) as f32;
            let cell_h = cell.height as f32 / term.height.max(1) as f32;

            if cell_w > 0.0 && cell_h > 0.0 {
                self.scale.height = self.scale.width * (cell_h / cell_w);
            }
        }

        // Keep some space for the UI
        self.cells = Size::new(term.width.max(1), term.height.max(2) - 1).cast();
        self.browser = self.cells.cast::<f32>().mul(self.scale).ceil().cast();

        self
    }
}

fn parse_var<T: FromStr>(var: &str) -> Option<T> {
    std::env::var(var).ok()?.parse().ok()
}
