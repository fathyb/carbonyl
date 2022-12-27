use std::io::{self, Read};

use crate::terminal::input::*;

pub fn setup() -> io::Result<()> {
    raw_tty::setup()
}

/// Listen for input events in stdin.
/// This will block, so it should run from a dedicated thread.
pub fn listen<T, F>(mut callback: F) -> io::Result<T>
where
    F: FnMut(Event) -> Option<T>,
{
    let mut buf = [0u8; 1024];
    let mut stdin = io::stdin();
    let mut parser = Parser::new();

    loop {
        // Wait for some input
        let size = stdin.read(&mut buf)?;
        let mut scroll = 0;

        // Parse the input for xterm commands
        for event in parser.parse(&buf[0..size]) {
            // Allow the callback to return early (ie. handle ctrl+c)
            if let Event::Scroll { delta } = event {
                scroll += delta;
            } else if let Some(result) = callback(event) {
                return Ok(result);
            }
        }

        if scroll != 0 {
            if let Some(result) = callback(Event::Scroll { delta: scroll }) {
                return Ok(result);
            }
        }
    }
}
