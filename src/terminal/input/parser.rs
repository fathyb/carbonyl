use crate::terminal::input::*;

#[derive(Clone)]
enum State {
    CharSequence,
    EscapeSequence,
    ControlSequence,
    MouseSequence(Mouse),
    DeviceControlSequence(DeviceControl),
}

pub struct Parser {
    state: State,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            state: State::CharSequence,
        }
    }

    pub fn parse(&mut self, input: &[u8]) -> Vec<Event> {
        let mut events = Vec::new();
        let mut state = self.state.clone();
        let mut emit = |e| {
            events.push(e);

            CharSequence
        };

        use Event::*;
        use State::*;

        for &key in input {
            state = match state {
                CharSequence => match key {
                    // ESC character, start an escape sequence
                    0x1b => EscapeSequence,
                    // CTRL-C pressed
                    0x03 => emit(Exit),
                    // Any other character should be parsed as text input
                    key => emit(KeyPress { key }),
                },
                EscapeSequence => match key {
                    // CSI
                    b'[' => ControlSequence,
                    // DCS
                    b'P' => DeviceControlSequence(DeviceControl::new()),
                    0x1b => {
                        emit(KeyPress { key: 0x1b });

                        continue;
                    }
                    key => {
                        emit(KeyPress { key: 0x1b });
                        emit(KeyPress { key })
                    }
                },
                ControlSequence => match key {
                    // Mouse input
                    b'<' => MouseSequence(Mouse::new()),
                    b'A' => emit(KeyPress { key: 0x26 }),
                    b'B' => emit(KeyPress { key: 0x28 }),
                    b'C' => emit(KeyPress { key: 0x27 }),
                    b'D' => emit(KeyPress { key: 0x25 }),
                    _ => CharSequence,
                },
                MouseSequence(ref mut mouse) => match key {
                    // Delimiter
                    b';' => {
                        if mouse.parse() == None {
                            CharSequence
                        } else {
                            continue;
                        }
                    }
                    // Terminator
                    b'm' | b'M' => {
                        if let Some(event) = Event::parse(mouse, key == b'm') {
                            emit(event)
                        } else {
                            CharSequence
                        }
                    }
                    // Consider anything else part of the value
                    key => {
                        mouse.push(key);

                        continue;
                    }
                },
                DeviceControlSequence(ref mut dcs) => match dcs.parse(key) {
                    DeviceControlEvent::Continue => continue,
                    DeviceControlEvent::Terminal(terminal) => emit(Terminal(terminal)),
                    _ => CharSequence,
                },
            }
        }

        self.state = state;

        events
    }
}
