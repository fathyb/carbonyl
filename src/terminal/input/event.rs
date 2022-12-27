use std::ops::BitAnd;

use super::Mouse;

#[derive(Debug)]
pub enum Event {
    KeyPress { key: u8 },
    MouseUp { row: usize, col: usize },
    MouseDown { row: usize, col: usize },
    MouseMove { row: usize, col: usize },
    Scroll { delta: isize },
    Exit,
}

impl Event {
    pub fn from(mouse: &mut Mouse, release: bool) -> Option<Event> {
        if !mouse.read() {
            return None;
        }

        match (mouse.btn, mouse.col, mouse.row) {
            (Some(btn), Some(col), Some(row)) => Some({
                if Mask::ScrollDown & btn {
                    Event::Scroll { delta: -1 }
                } else if Mask::ScrollUp & btn {
                    Event::Scroll { delta: 1 }
                } else {
                    let col = col as usize - 1;
                    let row = row as usize - 1;

                    if release {
                        Event::MouseUp { row, col }
                    } else if Mask::MouseMove & btn {
                        Event::MouseMove { row, col }
                    } else {
                        Event::MouseDown { row, col }
                    }
                }
            }),
            _ => None,
        }
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
