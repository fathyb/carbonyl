use std::io::{self, Stdout, Write};

use crate::gfx::{Color, Point};

use super::Cell;

#[derive(PartialEq)]
enum PaintMode {
    Text,
    Bitmap,
}

pub struct Painter {
    mode: PaintMode,
    output: Stdout,
    buffer: Vec<u8>,
    cursor: Option<Point<u32>>,
    true_color: bool,
    background: Option<Color>,
    foreground: Option<Color>,
    background_code: Option<u8>,
    foreground_code: Option<u8>,
}

impl Painter {
    pub fn new() -> Painter {
        Painter {
            mode: PaintMode::Text,
            buffer: Vec::new(),
            cursor: None,
            output: io::stdout(),
            true_color: if let Ok(value) = std::env::var("COLORTERM") {
                match value.as_str() {
                    "truecolor" | "24bit" => true,
                    _ => false,
                }
            } else {
                false
            },
            background: None,
            foreground: None,
            background_code: None,
            foreground_code: None,
        }
    }

    pub fn true_color(&self) -> bool {
        self.true_color
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.output.write(self.buffer.as_slice())?;
        self.output.flush()?;
        self.buffer.clear();

        Ok(())
    }

    pub fn paint(&mut self, cell: &Cell) -> io::Result<()> {
        let &Cell {
            cursor,
            ref grapheme,
            top: mut background,
            bottom: mut foreground,
        } = cell;

        let (char, width, escape) = if let Some(grapheme) = grapheme {
            if grapheme.index > 0 {
                return Ok(());
            }

            background = background.avg_with(foreground);
            foreground = grapheme.color;

            (
                grapheme.char.as_str(),
                grapheme.width as u32,
                if self.mode == PaintMode::Bitmap {
                    self.mode = PaintMode::Text;

                    Some("\x1b[22m\x1b[24m")
                } else {
                    None
                },
            )
        } else {
            (
                "▄",
                1,
                if self.mode == PaintMode::Text {
                    self.mode = PaintMode::Bitmap;

                    Some("\x1b[1m\x1b[4m")
                } else {
                    None
                },
            )
        };

        if self.cursor != Some(cursor) {
            write!(self.buffer, "\x1b[{};{}H", cursor.y + 1, cursor.x + 1)?;
        };

        self.cursor = Some(cursor + Point::new(width, 0));

        if self.background != Some(background) {
            self.background = Some(background);

            if self.true_color {
                write!(
                    self.buffer,
                    "\x1b[48;2;{};{};{}m",
                    background.r, background.g, background.b,
                )?
            } else {
                let code = background.to_xterm();

                if self.background_code != Some(code) {
                    self.background_code = Some(code);

                    write!(self.buffer, "\x1b[48;5;{}m", code)?
                }
            }
        }

        if self.foreground != Some(foreground) {
            self.foreground = Some(foreground);

            if self.true_color {
                write!(
                    self.buffer,
                    "\x1b[38;2;{};{};{}m",
                    foreground.r, foreground.g, foreground.b,
                )?
            } else {
                let code = foreground.to_xterm();

                if self.foreground_code != Some(code) {
                    self.foreground_code = Some(code);

                    write!(self.buffer, "\x1b[38;5;{}m", code)?
                }
            }
        }

        if let Some(escape) = escape {
            self.buffer.write_all(escape.as_bytes())?
        }

        self.buffer.write_all(char.as_bytes())?;

        Ok(())
    }
}
