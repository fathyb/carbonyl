use std::io::{self, Read};

use crate::input::*;

/// Listen for input events in stdin.
/// This will block, so it should run from a dedicated thread.
pub fn listen<F>(mut callback: F) -> io::Result<()>
where
    F: FnMut(Vec<Event>),
{
    let mut buf = [0u8; 1024];
    let mut stdin = io::stdin();
    let mut parser = Parser::new();

    loop {
        // Wait for some input
        let size = stdin.read(&mut buf)?;
        let read = parser.parse(&buf[0..size]);
        let mut scroll = 0;
        let mut events = Vec::with_capacity(read.len());

        for event in read {
            match event {
                Event::Exit => return Ok(()),
                Event::Scroll { delta } => scroll += delta,
                event => events.push(event),
            }
        }

        if scroll != 0 {
            events.push(Event::Scroll { delta: scroll })
        }

        callback(events)
    }
}
