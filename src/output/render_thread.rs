use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Instant,
};

use crate::cli::CommandLine;

use super::{FrameSync, Renderer};

/// Control a rendering thread that lazily starts.
/// This allows the `Bridge` struct to be used in places
/// where we do not expected the rendering thread to start.
pub struct RenderThread {
    thread: Option<(Sender<Message>, JoinHandle<()>)>,
    enabled: bool,
}

type RenderClosure = Box<dyn FnMut(&mut Renderer) + Send>;
enum Message {
    Run(RenderClosure),
    Shutdown,
}

impl RenderThread {
    pub fn new() -> Self {
        Self {
            thread: None,
            enabled: false,
        }
    }

    /// Enable the rendering thread.
    /// Allows the thread to be lazily initiated.
    pub fn enable(&mut self) {
        self.enabled = true
    }

    /// Stop the rendering thread.
    /// Returns a `JoinHandle` if a thread was started.
    pub fn stop(&mut self) -> Option<JoinHandle<()>> {
        self.enabled = false;
        self.send(Message::Shutdown);

        let (_, handle) = self.thread.take()?;

        Some(handle)
    }

    /// Run a closure on the rendering thread.
    pub fn render<F>(&mut self, run: F)
    where
        F: FnMut(&mut Renderer) + Send + 'static,
    {
        self.send(Message::Run(Box::new(run)))
    }

    /// Boot the rendering thread, contains a simple event loop.
    fn boot(rx: Receiver<Message>) {
        let cmd = CommandLine::parse();
        let mut sync = FrameSync::new(cmd.fps);
        let mut renderer = Renderer::new();
        let mut needs_render = false;

        loop {
            // Get a deadline for the next frame
            let deadline = sync.deadline();
            let mut wait = true;

            loop {
                let message = if wait {
                    // On the first iteration of this loop, we want to block indefinitely
                    // until we get a message, after which we schedule a render.
                    wait = false;

                    rx.recv().ok()
                } else {
                    // On subsequence iterations, we want to process a maximum number of events
                    // until the deadline for the next frame.
                    rx.recv_timeout(deadline - Instant::now()).ok()
                };

                // Wait for some events before the deadline
                match message {
                    // Timeout and no message, render if needed
                    None => break,
                    // Shutdown the thread
                    Some(Message::Shutdown) => return,
                    // Run a closure and schedule a render
                    Some(Message::Run(mut closure)) => {
                        closure(&mut renderer);

                        needs_render = true;
                    }
                }
            }

            // Render if needed
            if needs_render {
                needs_render = false;

                // Update the frame sync timings
                sync.start();
                renderer.render().unwrap();
            }
        }
    }

    /// Send a message to the rendering thread.
    /// Creates a new thread if enabled and needed.
    fn send(&mut self, message: Message) {
        if let Some((tx, _)) = &self.thread {
            tx.send(message).unwrap()
        } else if self.enabled {
            let (tx, rx) = mpsc::channel();

            tx.send(message).unwrap();

            self.thread = Some((tx.clone(), thread::spawn(move || Self::boot(rx))));
        }
    }
}
