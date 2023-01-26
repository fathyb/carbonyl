use crate::{control_flow, input::ParseControlFlow};

use super::{resource::*, status::*};

#[derive(Default, Clone)]
enum Sequence {
    #[default]
    Code,
    Type(u8),
    Status(StatusParser),
    Resource(ResourceParser),
}

#[derive(Default, Clone)]
pub struct DeviceControl {
    sequence: Sequence,
}

impl DeviceControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&mut self, key: u8) -> ParseControlFlow {
        use Sequence::*;

        match self.sequence {
            Code => match key {
                b'0' | b'1' => control_flow!(self.sequence = Type(key); continue),
                _ => control_flow!(break),
            },
            Type(code) => match key {
                b'$' => {
                    control_flow!(self.sequence = Status(StatusParser::new(code)); continue)
                }
                b'+' => {
                    control_flow!(self.sequence = Resource(ResourceParser::new(code)); continue)
                }
                _ => control_flow!(break),
            },
            Status(ref mut status) => status.parse(key),
            Resource(ref mut resource) => resource.parse(key),
        }
    }
}
