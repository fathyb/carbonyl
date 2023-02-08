use std::io::{self, Read};

use crate::input::*;

/// Listen for input events in stdin.
/// This will block, so it should run from a dedicated thread.
pub fn listen<F>(mut callback: F) -> io::Result<()>
where
    F: FnMut(Event),
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
            match event {
                Event::Exit => return Ok(()),
                Event::Scroll { delta } => scroll += delta,
                event => callback(event),
            }
        }

        if scroll != 0 {
            callback(Event::Scroll { delta: scroll })
        }
    }
}
