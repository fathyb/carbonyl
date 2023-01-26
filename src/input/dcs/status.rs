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
        match self.sequence {
            Sequence::Start => match key {
                b'r' => control_flow!(self.sequence = Sequence::Value; continue),
                _ => control_flow!(break),
            },
            Sequence::Value => match key {
                b';' | 0x1b => {
                    if key == 0x1b {
                        self.op = self.buffer.pop();
                        self.sequence = Sequence::Terminator;
                    }

                    if let Ok(str) = String::from_utf8(std::mem::take(&mut self.buffer)) {
                        self.values.push(str);
                    }

                    control_flow!(continue)
                }
                key => control_flow!(self.buffer.push(key); continue),
            },
            Sequence::Terminator => {
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
                            return control_flow!(
                                break Event::Terminal(TerminalEvent::TrueColorSupported)
                            );
                        }
                    }
                }

                control_flow!(break)
            }
        }
    }
}
