use super::Event;

#[derive(Debug)]
pub struct Mouse {
    pub buf: Option<Vec<u8>>,
    pub btn: Option<u32>,
    pub col: Option<u32>,
    pub row: Option<u32>,
}

impl Mouse {
    pub fn start(&mut self) {
        self.buf = Some(Vec::new())
    }

    pub fn end(&mut self, key: u8, events: &mut Vec<Event>) {
        if let Some(event) = Event::from(self, key == 0x6d) {
            events.push(event)
        }
    }

    pub fn reset(&mut self) {
        self.buf = None;
        self.btn = None;
        self.col = None;
        self.row = None;
    }

    pub fn read(&mut self) -> bool {
        if self.parse() == None {
            self.reset();

            false
        } else {
            true
        }
    }

    fn parse(&mut self) -> Option<()> {
        if let Some(ref buf) = self.buf {
            let string = std::str::from_utf8(buf).ok()?;
            let data = string.parse().ok()?;

            self.update(data)
        } else {
            None
        }
    }

    fn update(&mut self, data: u32) -> Option<()> {
        if self.btn == None {
            self.btn = Some(data)
        } else if self.col == None {
            self.col = Some(data)
        } else if self.row == None {
            self.row = Some(data)
        } else {
            return None;
        }

        self.buf = Some(Vec::new());

        return Some(());
    }
}
