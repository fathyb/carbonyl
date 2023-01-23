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
        let mut emit = |e| events.push(e);
        let mut state = self.state.clone();

        use Event::*;
        use State::*;

        for &key in input {
            match state {
                CharSequence => match key {
                    // ESC character, start an escape sequence
                    0x1b => state = EscapeSequence,
                    // CTRL-C pressed
                    0x03 => emit(Exit),
                    // Any other character should be parsed as text input
                    key => emit(KeyPress { key }),
                },
                EscapeSequence => match key {
                    // CSI
                    b'[' => state = ControlSequence,
                    // DCS
                    b'P' => state = DeviceControlSequence(DeviceControl::new()),
                    key => {
                        // Unrecognized sequence, emit an escape keypress
                        emit(KeyPress { key: 0x1b });

                        // If this isn't an escape character, emit a
                        // keypress for this key and close the sequence
                        if key != 0x1b {
                            state = CharSequence;

                            emit(KeyPress { key });
                        }
                    }
                },
                ControlSequence => match key {
                    // Mouse input
                    b'<' => state = MouseSequence(Mouse::new()),
                    _ => {
                        state = CharSequence;

                        match key {
                            // Map arrow keys events to key codes
                            b'A' => emit(KeyPress { key: 0x26 }),
                            b'B' => emit(KeyPress { key: 0x28 }),
                            b'C' => emit(KeyPress { key: 0x27 }),
                            b'D' => emit(KeyPress { key: 0x25 }),
                            // Ignore anything else
                            _ => continue,
                        }
                    }
                },
                MouseSequence(ref mut mouse) => match key {
                    // Delimiter
                    b';' => {
                        if mouse.parse() == None {
                            state = CharSequence
                        }
                    }
                    // Terminator
                    b'm' | b'M' => {
                        if let Some(event) = Event::parse(mouse, key == b'm') {
                            emit(event)
                        }

                        state = CharSequence
                    }
                    // Consider anything else part of the value
                    key => mouse.push(key),
                },
                DeviceControlSequence(ref mut dcs) => match dcs.parse(key) {
                    DeviceControlEvent::Continue => continue,
                    event => {
                        state = CharSequence;

                        match event {
                            DeviceControlEvent::TerminalName(name) => {
                                emit(Terminal(TerminalEvent::Name(name)))
                            }
                            DeviceControlEvent::TrueColorSupported => {
                                emit(Terminal(TerminalEvent::TrueColorSupported))
                            }
                            DeviceControlEvent::Break | DeviceControlEvent::Continue => continue,
                        }
                    }
                },
            }
        }

        self.state = state;

        events
    }
}
