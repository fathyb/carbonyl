use crate::control_flow;

use super::{Event, ParseControlFlow, TerminalEvent};

#[derive(Default, Clone, Debug)]
pub struct Graphics {
    params: Vec<u32>,
    buffer: Vec<u8>,
}

impl Graphics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&mut self, key: u8) -> ParseControlFlow {
        match key {
            b'0'..=b'9' => {
                self.buffer.push(key);

                control_flow!(continue)
            }
            b';' => {
                self.push_param();

                control_flow!(continue)
            }
            b'S' => {
                self.push_param();

                control_flow!(break self.event())?
            }
            _ => control_flow!(break)?,
        }
    }

    fn push_param(&mut self) {
        if self.buffer.is_empty() {
            self.params.push(0);
            return;
        }

        if let Ok(text) = std::str::from_utf8(&self.buffer) {
            if let Ok(value) = text.parse::<u32>() {
                self.params.push(value);
            }
        }

        self.buffer.clear();
    }

    fn event(&mut self) -> Option<Event> {
        let params = std::mem::take(&mut self.params);

        if params.len() >= 2 {
            let item = params[0];
            let status = params[1];

            if item == 2 && status == 0 {
                let width = params.get(2).copied().unwrap_or_default();
                let height = params.get(3).copied().unwrap_or_default();

                return Some(Event::Terminal(TerminalEvent::SixelSupported {
                    width,
                    height,
                }));
            }
        }

        None
    }
}
