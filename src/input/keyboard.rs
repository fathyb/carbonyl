use crate::control_flow;

use super::{Event, Key, ParseControlFlow};

enum State {
    Separator,
    Modifier,
    Key,
}

pub struct Keyboard {
    alt: bool,
    state: State,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            alt: false,
            state: State::Separator,
        }
    }
    pub fn key(key: u8, alt: bool) -> Option<Event> {
        match key {
            // Up
            b'A' => Some(Event::KeyPress {
                key: Key { char: 0x11, alt },
            }),
            // Down
            b'B' => Some(Event::KeyPress {
                key: Key { char: 0x12, alt },
            }),
            // Right
            b'C' => Some(Event::KeyPress {
                key: Key { char: 0x13, alt },
            }),
            // Left
            b'D' => Some(Event::KeyPress {
                key: Key { char: 0x14, alt },
            }),
            _ => None,
        }
    }

    pub fn parse(&mut self, key: u8) -> ParseControlFlow {
        self.state = match self.state {
            State::Separator => match key {
                b';' => State::Modifier,
                _ => control_flow!(break)?,
            },
            State::Modifier => {
                if key == b'3' {
                    self.alt = true;
                }

                State::Key
            }
            State::Key => control_flow!(break Self::key(key, self.alt))?,
        };

        control_flow!(continue)
    }
}
