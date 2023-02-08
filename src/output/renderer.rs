use std::{
    io::{self, Write},
    rc::Rc,
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    gfx::{Color, Point, Rect, Size},
    input::Key,
    ui::navigation::{Navigation, NavigationAction},
};

use super::{Cell, Grapheme, Painter};

struct Dimensions {
    /// Size of a terminal cell in pixels
    cell: Size,
    /// Size of the browser window in pixels
    browser: Size,
    /// Size of the terminal window in cells
    terminal: Size,
}

pub struct Renderer {
    nav: Navigation,
    cells: Vec<(Cell, Cell)>,
    dimensions: Dimensions,
    painter: Painter,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            nav: Navigation::new(),
            cells: Vec::with_capacity(0),
            painter: Painter::new(),
            dimensions: Dimensions {
                cell: Size::new(7, 14),
                browser: Size::new(0, 0),
                terminal: Size::new(0, 0),
            },
        }
    }

    pub fn enable_true_color(&mut self) {
        self.painter.set_true_color(true)
    }

    pub fn keypress(&mut self, key: &Key) -> io::Result<NavigationAction> {
        let action = self.nav.keypress(key);

        self.render()?;

        Ok(action)
    }
    pub fn mouse_up(&mut self, origin: Point) -> io::Result<NavigationAction> {
        let action = self.nav.mouse_up(origin);

        self.render()?;

        Ok(action)
    }
    pub fn mouse_down(&mut self, origin: Point) -> io::Result<NavigationAction> {
        let action = self.nav.mouse_down(origin);

        self.render()?;

        Ok(action)
    }
    pub fn mouse_move(&mut self, origin: Point) -> io::Result<NavigationAction> {
        let action = self.nav.mouse_move(origin);

        self.render()?;

        Ok(action)
    }

    pub fn push_nav(&mut self, url: &str, can_go_back: bool, can_go_forward: bool) {
        self.nav.push(url, can_go_back, can_go_forward)
    }

    pub fn get_size(&self) -> Size {
        self.dimensions.terminal
    }

    pub fn set_size(&mut self, cell: Size, terminal: Size) {
        let size = (terminal.width + terminal.width * terminal.height) as usize;

        self.nav.set_size(terminal);
        self.dimensions.cell = cell;
        self.dimensions.terminal = terminal;
        self.dimensions.browser = cell * terminal;

        let mut x = 0;
        let mut y = 0;
        let bound = terminal.width - 1;

        self.cells.clear();
        self.cells.resize_with(size, || {
            let cell = (Cell::new(x, y), Cell::new(x, y));

            if x < bound {
                x += 1;
            } else {
                x = 0;
                y += 1;
            }

            cell
        });
    }

    pub fn render(&mut self) -> io::Result<()> {
        let size = self.dimensions.terminal;

        for (origin, element) in self.nav.render(size) {
            self.fill_rect(
                Rect::new(origin.x, origin.y, element.text.width() as u32, 1),
                element.background,
            );
            self.draw_text(&element.text, origin, Size::splat(0), element.foreground);
        }

        self.painter.begin()?;

        for (previous, current) in self.cells.iter_mut() {
            if current == previous {
                continue;
            }

            previous.top = current.top;
            previous.bottom = current.bottom;
            previous.grapheme = current.grapheme.clone();

            self.painter.paint(current)?;
        }

        self.painter.end(self.nav.cursor())?;

        Ok(())
    }

    /// Draw the background from a pixel array encoded in RGBA8888
    pub fn draw_background(&mut self, pixels: &mut [u8], rect: Rect) -> io::Result<()> {
        let viewport = self.dimensions.terminal.cast::<usize>();
        let pixels_row = viewport.width * 4;

        if pixels.len() != pixels_row * viewport.height * 2 {
            return Ok(());
        }

        let pos = rect.origin.cast::<usize>() / (1, 2);
        let size = rect.size.cast::<usize>() / (1, 2);
        let pixels_left = pos.x * 4;
        let pixels_width = size.width * 4;

        // Iterate over each row
        for y in pos.y..pos.y + size.height {
            // Terminal chars have an aspect ratio of 2:1.
            // In order to display perfectly squared pixels, we
            // render a unicode glyph taking the bottom half of the cell
            // using a foreground representing the bottom pixel,
            // and a background representing the top pixel.
            // This means that the pixel input buffer should be twice the size
            // of the terminal cell buffer (two pixels take one terminal cell).
            let left = pixels_left + y * 2 * pixels_row;
            let right = left + pixels_width;
            // Get a slice pointing to the top pixel row
            let mut top_row = pixels[left..right].iter();
            // Get a slice pointing to the bottom pixel row
            let mut bottom_row = pixels[left + pixels_row..right + pixels_row].iter();
            let cells_left = y * viewport.width + pos.x + viewport.width;
            let cells = self.cells[cells_left..].iter_mut();

            // Iterate over each column
            for (_, cell) in cells {
                match (
                    Color::from_iter(&mut top_row),
                    Color::from_iter(&mut bottom_row),
                ) {
                    (Some(top), Some(bottom)) => {
                        cell.top = top;
                        cell.bottom = bottom;
                    }
                    _ => break,
                }
            }
        }

        self.render()
    }

    pub fn clear_text(&mut self) {
        for (_, cell) in self.cells.iter_mut() {
            cell.grapheme = None
        }
    }

    pub fn set_title(&self, title: &str) -> io::Result<()> {
        let mut stdout = io::stdout();

        write!(stdout, "\x1b]0;{title}\x07")?;
        write!(stdout, "\x1b]1;{title}\x07")?;
        write!(stdout, "\x1b]2;{title}\x07")?;

        stdout.flush()
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.draw(rect, |cell| {
            cell.top = color;
            cell.bottom = color;
            cell.grapheme = None;
        })
    }

    pub fn draw<F>(&mut self, bounds: Rect, mut draw: F)
    where
        F: FnMut(&mut Cell),
    {
        let origin = bounds.origin.cast::<usize>();
        let size = bounds.size.cast::<usize>();
        let viewport_width = self.dimensions.terminal.width as usize;
        let top = origin.y;
        let bottom = top + size.height;

        // Iterate over each row
        for y in top..bottom {
            let left = y * viewport_width + origin.x;
            let right = left + size.width;

            for (_, current) in self.cells[left..right].iter_mut() {
                draw(current)
            }
        }
    }

    /// Render some text into the terminal output
    pub fn draw_text(&mut self, string: &str, origin: Point, size: Size, color: Color) {
        // Get an iterator starting at the text origin
        let len = self.cells.len();
        let viewport = &self.dimensions.terminal;

        if size.width > 2 && size.height > 2 {
            let x = origin.x.max(0).min(viewport.width as i32);
            let top = (origin.y + 1).max(0);
            let bottom = top + size.height as i32;

            for y in top..bottom {
                let index = x + y / 2 * (viewport.width as i32);
                let left = len.min(index as usize);
                let right = len.min(left + size.width as usize);

                for (_, cell) in self.cells[left..right].iter_mut() {
                    cell.grapheme = None
                }
            }
        } else {
            // Compute the buffer index based on the position
            let index = origin.x + (origin.y + 1) / 2 * (viewport.width as i32);
            let mut iter = self.cells[len.min(index as usize)..].iter_mut();

            // Get every Unicode grapheme in the input string
            for grapheme in UnicodeSegmentation::graphemes(string, true) {
                let width = grapheme.width();

                for index in 0..width {
                    // Get the next terminal cell at the given position
                    match iter.next() {
                        // Stop if we're at the end of the buffer
                        None => return,
                        // Set the cell to the current grapheme
                        Some((_, cell)) => {
                            let next = Grapheme {
                                // Create a new shared reference to the text
                                color,
                                index,
                                width,
                                // Export the set of unicode code points for this graphene into an UTF-8 string
                                char: grapheme.to_string(),
                            };

                            if match cell.grapheme {
                                None => true,
                                Some(ref previous) => {
                                    previous.color != next.color || previous.char != next.char
                                }
                            } {
                                cell.grapheme = Some(Rc::new(next))
                            }
                        }
                    }
                }
            }
        }
    }
}
