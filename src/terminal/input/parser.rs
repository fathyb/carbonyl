use crate::terminal::input::*;

pub struct Parser {
    esc: bool,
    csi: bool,
    mouse: Mouse,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            esc: false,
            csi: false,
            mouse: Mouse {
                buf: None,
                btn: None,
                col: None,
                row: None,
            },
        }
    }

    pub fn parse(&mut self, input: &[u8]) -> Vec<Event> {
        let mut events = Vec::new();
        let Parser {
            mut esc,
            mut csi,
            ref mut mouse,
        } = self;

        for &key in input {
            if esc {
                // Inside an escape sequence
                if let Some(ref mut buf) = mouse.buf {
                    // Process mouse input sequence
                    match key {
                        // Delimiter: concat characters, parse number, and clear buffer
                        0x3b => esc = mouse.read(),
                        // Terminator: emit mouse move, movement, or release based on terminator
                        0x4d | 0x6d => {
                            mouse.end(key, &mut events);

                            esc = false
                        }
                        // Consider anything else part of the value
                        _ => buf.push(key),
                    }
                } else if csi {
                    // Inside a control sequence
                    match key {
                        // Mouse input
                        0x3c => mouse.start(),
                        // Map arrow keys events to key codes
                        0x41..=0x44 => events.push(Event::KeyPress {
                            key: [0x26, 0x28, 0x27, 0x25][(key - 0x41) as usize],
                        }),
                        // Ignore anything else
                        _ => esc = false,
                    }
                } else if key == 0x5b {
                    // [ character, start a CSI sequence
                    csi = true
                } else {
                    // Unrecognized sequence, emit an ESC keypress
                    events.push(Event::KeyPress { key: 0x1b });

                    if key != 0x1b {
                        // Cancel the sequence only if this isn't an ESC character
                        esc = false;

                        events.push(Event::KeyPress { key });
                    }
                }
            } else if key == 0x1b {
                // ESC character, start an escape sequence
                esc = true;
                csi = false;
                mouse.reset();
            } else if key == 0x03 {
                // CTRL-C pressed
                events.push(Event::Exit)
            } else {
                // Any other character should be parser as text input
                events.push(Event::KeyPress { key })
            }
        }

        self.esc = esc;
        self.csi = csi;

        events
    }
}
