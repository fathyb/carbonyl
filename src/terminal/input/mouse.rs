#[derive(Clone, Debug)]
pub struct Mouse {
    pub btn: Option<u32>,
    pub col: Option<u32>,
    pub row: Option<u32>,

    buf: Vec<u8>,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            btn: None,
            col: None,
            row: None,
            buf: Vec::new(),
        }
    }

    pub fn push(&mut self, char: u8) {
        self.buf.push(char)
    }

    pub fn parse(&mut self) -> Option<()> {
        let buf = std::mem::take(&mut self.buf);
        let str = std::str::from_utf8(&buf).ok()?;
        let num = Some(str.parse().ok()?);

        match (self.btn, self.col, self.row) {
            (None, _, _) => self.btn = num,
            (_, None, _) => self.col = num,
            (_, _, None) => self.row = num,
            _ => return None,
        }

        return Some(());
    }
}
