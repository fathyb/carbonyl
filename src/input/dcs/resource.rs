use crate::{
    control_flow,
    input::{Event, ParseControlFlow, TerminalEvent},
};

#[derive(Copy, Clone)]
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

        self.sequence = match self.sequence {
            Start => match key {
                b'r' => Name,
                _ => control_flow!(break)?,
            },
            Name => match key {
                0x1b => Terminator,
                b'=' => Value,
                key => self.push_char(key),
            },
            Value => match key {
                0x1b => Terminator,
                key => self.push_char(key),
            },
            Terminator => control_flow!(break self.parse_event(key))?,
        };

        control_flow!(continue)
    }

    fn push_char(&mut self, key: u8) -> Sequence {
        match self.sequence {
            Sequence::Name => self.name.push(key),
            Sequence::Value => self.value.push(key),
            _ => (),
        }

        self.sequence
    }

    fn parse_event(&self, key: u8) -> Option<Event> {
        if key == b'\\' && self.code == b'1' {
            let name = read_hex_string(self.name.as_slice());
            let value = read_hex_string(self.value.as_slice());

            if let (Some(name), Some(value)) = (name, value) {
                if name == "TN" {
                    return Some(Event::Terminal(TerminalEvent::Name(value)));
                }
            }
        }

        None
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
