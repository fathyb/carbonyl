use std::ops::{BitAnd, ControlFlow};

use super::{Event, ParseControlFlow};

#[derive(Default, Clone, Debug)]
pub struct Mouse {
    buf: Vec<u8>,
    btn: Option<u32>,
    col: Option<u32>,
    row: Option<u32>,
}

impl Mouse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&mut self, key: u8) -> ParseControlFlow {
        match key {
            b';' => match self.read() {
                None => ControlFlow::Break(None),
                Some(()) => ControlFlow::Continue(None),
            },
            b'm' | b'M' => ControlFlow::Break(self.get(key)),
            key => {
                self.buf.push(key);

                ControlFlow::Continue(None)
            }
        }
    }

    fn read(&mut self) -> Option<()> {
        let buf = std::mem::take(&mut self.buf);
        let str = std::str::from_utf8(&buf).ok()?;
        let num = Some(str.parse().ok()?);

        match (self.btn, self.col, self.row) {
            (None, _, _) => self.btn = num,
            (_, None, _) => self.col = num,
            (_, _, None) => self.row = num,
            _ => {
                eprintln!("Misformed mouse sequence");

                return None;
            }
        }

        return Some(());
    }

    fn get(&mut self, key: u8) -> Option<Event> {
        let (btn, col, row) = {
            self.read()?;

            (self.btn?, self.col?, self.row?)
        };

        Some({
            if Mask::ScrollDown & btn {
                Event::Scroll { delta: -1 }
            } else if Mask::ScrollUp & btn {
                Event::Scroll { delta: 1 }
            } else {
                let col = col as usize - 1;
                let row = row as usize - 1;

                if key == b'm' {
                    Event::MouseUp { row, col }
                } else if Mask::MouseMove & btn {
                    Event::MouseMove { row, col }
                } else {
                    Event::MouseDown { row, col }
                }
            }
        })
    }
}

enum Mask {
    MouseMove = 0x20,
    ScrollUp = 0x40,
    ScrollDown = 0x41,
}

impl BitAnd<u32> for Mask {
    type Output = bool;

    fn bitand(self, rhs: u32) -> bool {
        let mask = self as u32;

        mask & rhs == mask
    }
}
