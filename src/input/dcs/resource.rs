use crate::{
    control_flow,
    input::{Event, ParseControlFlow, TerminalEvent},
};

#[derive(Clone)]
enum Sequence {
    Start,
    Name,
    Value,
    Terminator,
}

#[derive(Clone)]
pub struct ResourceParser {
    code: u8,
    sequence: Sequence,
    name: Vec<u8>,
    value: Vec<u8>,
}

impl ResourceParser {
    pub fn new(code: u8) -> Self {
        Self {
            code,
            sequence: Sequence::Start,
            name: Vec::new(),
            value: Vec::new(),
        }
    }

    pub fn parse(&mut self, key: u8) -> ParseControlFlow {
        use Sequence::*;

        match self.sequence {
            Start => match key {
                b'r' => control_flow!(self.sequence = Name; continue),
                _ => control_flow!(break),
            },
            Name => match key {
                0x1b => control_flow!(self.sequence = Terminator; continue),
                b'=' => control_flow!(self.sequence = Value; continue),
                key => control_flow!(self.name.push(key); continue),
            },
            Value => match key {
                0x1b => control_flow!(self.sequence = Terminator; continue),
                key => control_flow!(self.value.push(key); continue),
            },
            Terminator => {
                if key == b'\\' && self.code == b'1' {
                    let name = read_hex_string(self.name.as_slice());
                    let value = read_hex_string(self.value.as_slice());

                    if let (Some(name), Some(value)) = (name, value) {
                        if name == "TN" {
                            return control_flow!(
                                break Event::Terminal(TerminalEvent::Name(value))
                            );
                        }
                    }
                }

                control_flow!(break)
            }
        }
    }
}

fn read_hex_string(str: &[u8]) -> Option<String> {
    let mut iter = str.into_iter();
    let mut vec = Vec::with_capacity(str.len() / 2);

    loop {
        match (iter.next(), iter.next()) {
            (Some(left), Some(right)) => {
                let chunk = [*left, *right];
                let hex = std::str::from_utf8(&chunk).ok()?;

                vec.push(u8::from_str_radix(hex, 16).ok()?)
            }
            _ => break,
        }
    }

    Some(std::str::from_utf8(&vec).ok()?.to_owned())
}
