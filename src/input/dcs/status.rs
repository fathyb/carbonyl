use crate::{
    control_flow,
    input::{Event, ParseControlFlow, TerminalEvent},
};

#[derive(Default, Clone)]
enum Sequence {
    #[default]
    Start,
    Value,
    Terminator,
}

#[derive(Default, Clone)]
pub struct StatusParser {
    code: u8,
    op: Option<u8>,
    sequence: Sequence,
    buffer: Vec<u8>,
    values: Vec<String>,
}

impl StatusParser {
    pub fn new(code: u8) -> Self {
        let mut parser = Self::default();

        parser.code = code;

        parser
    }

    pub fn parse(&mut self, key: u8) -> ParseControlFlow {
        use Sequence::*;

        self.sequence = match self.sequence {
            Start => match key {
                b'r' => Value,
                _ => control_flow!(break)?,
            },
            Value => match key {
                0x1b => self.terminate(),
                b';' => self.push_value(),
                char => self.push_char(char),
            },
            Terminator => control_flow!(break self.parse_event(key))?,
        };

        control_flow!(continue)
    }

    fn terminate(&mut self) -> Sequence {
        self.op = self.buffer.pop();

        self.push_value();

        Sequence::Terminator
    }

    fn push_char(&mut self, key: u8) -> Sequence {
        self.buffer.push(key);

        Sequence::Value
    }

    fn push_value(&mut self) -> Sequence {
        if let Ok(str) = String::from_utf8(std::mem::take(&mut self.buffer)) {
            self.values.push(str);
        }

        Sequence::Value
    }

    fn parse_event(&self, key: u8) -> Option<Event> {
        if key == b'\\' && self.code == b'1' && self.op == Some(b'm') {
            for value in &self.values {
                let mut val = 0;
                let mut set = Vec::new();

                for &char in value.as_bytes() {
                    match char {
                        b'0'..=b'9' => val = val * 10 + char - b'0',
                        b':' => set.push(std::mem::take(&mut val)),
                        _ => break,
                    }
                }

                set.push(val);

                if set.len() > 4 && set[1] == 2 && (set[0] == 38 || set[0] == 48) {
                    return Some(Event::Terminal(TerminalEvent::TrueColorSupported));
                }
            }
        }

        None
    }
}
