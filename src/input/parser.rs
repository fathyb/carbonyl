use std::ops::ControlFlow;

use crate::input::*;

#[derive(Default)]
pub struct Parser {
    events: Vec<Event>,
    sequence: Sequence,
}

#[derive(Default)]
enum Sequence {
    #[default]
    Char,
    Escape,
    Control,
    Mouse(Mouse),
    DeviceControl(DeviceControl),
}

#[derive(Debug)]
pub enum TerminalEvent {
    Name(String),
    TrueColorSupported,
}

#[derive(Debug)]
pub enum Event {
    KeyPress { key: u8 },
    MouseUp { row: usize, col: usize },
    MouseDown { row: usize, col: usize },
    MouseMove { row: usize, col: usize },
    Scroll { delta: isize },
    Terminal(TerminalEvent),
    Exit,
}

pub type ParseControlFlow = ControlFlow<Option<Event>, Option<Event>>;

impl Parser {
    pub fn new() -> Parser {
        Self::default()
    }

    pub fn parse(&mut self, input: &[u8]) -> Vec<Event> {
        let mut sequence = std::mem::take(&mut self.sequence);

        macro_rules! emit {
            ($event:expr) => {{
                self.events.push($event);
                Sequence::Char
            }};
            ($event:expr; continue) => {{
                self.events.push($event);
                continue;
            }};
        }
        macro_rules! parse {
            ($parser:expr, $key:expr) => (
                match $parser.parse($key) {
                    ControlFlow::Break(None) => Sequence::Char,
                    ControlFlow::Break(Some(event)) => emit!(event),
                    ControlFlow::Continue(None) => continue,
                    ControlFlow::Continue(Some(event)) => emit!(event; continue),
                }
            );
        }

        for &key in input {
            sequence = match sequence {
                Sequence::Char => match key {
                    0x1b => Sequence::Escape,
                    0x03 => emit!(Event::Exit),
                    key => emit!(Event::KeyPress { key }),
                },
                Sequence::Escape => match key {
                    b'[' => Sequence::Control,
                    b'P' => Sequence::DeviceControl(DeviceControl::new()),
                    0x1b => emit!(Event::KeyPress { key: 0x1b }; continue),
                    key => {
                        emit!(Event::KeyPress { key: 0x1b });
                        emit!(Event::KeyPress { key })
                    }
                },
                Sequence::Control => match key {
                    b'<' => Sequence::Mouse(Mouse::new()),
                    // Up
                    b'A' => emit!(Event::KeyPress { key: 0x11 }),
                    // Down
                    b'B' => emit!(Event::KeyPress { key: 0x12 }),
                    // Right
                    b'C' => emit!(Event::KeyPress { key: 0x13 }),
                    // Left
                    b'D' => emit!(Event::KeyPress { key: 0x14 }),
                    _ => Sequence::Char,
                },
                Sequence::Mouse(ref mut mouse) => parse!(mouse, key),
                Sequence::DeviceControl(ref mut dcs) => parse!(dcs, key),
            }
        }

        self.sequence = sequence;

        std::mem::take(&mut self.events)
    }
}
