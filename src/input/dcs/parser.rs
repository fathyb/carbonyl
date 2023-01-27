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

        self.sequence = match self.sequence {
            Code => match key {
                b'0' | b'1' => Type(key),
                _ => control_flow!(break)?,
            },
            Type(code) => match key {
                b'$' => Status(StatusParser::new(code)),
                b'+' => Resource(ResourceParser::new(code)),
                _ => control_flow!(break)?,
            },
            Status(ref mut status) => return status.parse(key),
            Resource(ref mut resource) => return resource.parse(key),
        };

        control_flow!(continue)
    }
}
