use super::TerminalEvent;

#[derive(Clone)]
enum State {
    Code,
    Type(u8),
    Status(DeviceControlStatus),
    Resource(DeviceControlResource),
}

#[derive(Clone)]
pub struct DeviceControl {
    state: State,
}

pub enum DeviceControlEvent {
    Break,
    Continue,
    Terminal(TerminalEvent),
    TrueColorSupported,
}

impl DeviceControl {
    pub fn new() -> Self {
        DeviceControl { state: State::Code }
    }

    pub fn parse(&mut self, key: u8) -> DeviceControlEvent {
        use DeviceControlEvent::*;
        use State::*;

        match self.state {
            Code => match key {
                b'0' | b'1' => self.state = Type(key),
                _ => return Break,
            },
            Type(code) => match key {
                b'$' => self.state = Status(DeviceControlStatus::new(code)),
                b'+' => self.state = Resource(DeviceControlResource::new(code)),
                _ => return Break,
            },
            Status(ref mut status) => return status.parse(key),
            Resource(ref mut resource) => return resource.parse(key),
        }

        Continue
    }
}

#[derive(Clone)]
enum DeviceControlResourceState {
    Start,
    Name,
    Value,
    Terminator,
}

#[derive(Clone)]
struct DeviceControlResource {
    code: u8,
    state: DeviceControlResourceState,
    name: Vec<u8>,
    value: Vec<u8>,
}

impl DeviceControlResource {
    fn new(code: u8) -> Self {
        Self {
            code,
            state: DeviceControlResourceState::Start,
            name: Vec::new(),
            value: Vec::new(),
        }
    }

    fn parse(&mut self, key: u8) -> DeviceControlEvent {
        use DeviceControlEvent::*;
        use DeviceControlResourceState::*;

        match self.state {
            Start => match key {
                b'r' => self.state = Name,
                _ => return Break,
            },
            Name => match key {
                0x1b => self.state = Terminator,
                b'=' => self.state = Value,
                key => self.name.push(key),
            },
            Value => match key {
                0x1b => self.state = Terminator,
                key => self.value.push(key),
            },
            Terminator => {
                if key == b'\\' && self.code == b'1' {
                    let name = read_hex_string(self.name.as_slice());
                    let value = read_hex_string(self.value.as_slice());

                    if let (Some(name), Some(value)) = (name, value) {
                        if name == "TN" {
                            return Terminal(TerminalEvent::Name(value));
                        }
                    }
                }

                return Break;
            }
        }

        Continue
    }
}

#[derive(Clone)]
enum DeviceControlStatusState {
    Start,
    Value,
    Terminator,
}

#[derive(Clone)]
struct DeviceControlStatus {
    code: u8,
    op: Option<u8>,
    state: DeviceControlStatusState,
    buffer: Vec<u8>,
    values: Vec<String>,
}

impl DeviceControlStatus {
    fn new(code: u8) -> Self {
        Self {
            code,
            op: None,
            state: DeviceControlStatusState::Start,
            buffer: Vec::new(),
            values: Vec::new(),
        }
    }

    fn parse(&mut self, key: u8) -> DeviceControlEvent {
        use DeviceControlEvent::*;
        use DeviceControlStatusState::*;

        match self.state {
            Start => match key {
                b'r' => self.state = Value,
                _ => return Break,
            },
            Value => match key {
                b';' | 0x1b => {
                    if key == 0x1b {
                        self.op = self.buffer.pop();
                        self.state = Terminator;
                    }

                    if let Ok(str) = String::from_utf8(std::mem::take(&mut self.buffer)) {
                        self.values.push(str);
                    }
                }
                key => self.buffer.push(key),
            },
            Terminator => {
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
                            return Terminal(TerminalEvent::TrueColorSupported);
                        }
                    }
                }

                return Break;
            }
        }

        Continue
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
