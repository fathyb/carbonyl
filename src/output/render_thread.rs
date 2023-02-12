use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::cli::CommandLine;

use super::{FrameSync, Renderer};

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

    pub fn enable(&mut self) {
        self.enabled = true
    }

    pub fn stop(&mut self) -> Option<JoinHandle<()>> {
        self.enabled = false;
        self.send(Message::Shutdown);

        let (_, handle) = self.thread.take()?;

        Some(handle)
    }

    pub fn run<F>(&mut self, run: F)
    where
        F: FnMut(&mut Renderer) + Send + 'static,
    {
        self.send(Message::Run(Box::new(run)))
    }

    fn send(&mut self, message: Message) {
        if let Some((tx, _)) = &self.thread {
            tx.send(message).unwrap()
        } else if self.enabled {
            let (tx, rx) = mpsc::channel();

            tx.send(message).unwrap();

            self.thread = Some((
                tx.clone(),
                thread::spawn(move || {
                    let cmd = CommandLine::parse();
                    let mut sync = FrameSync::new(cmd.fps);
                    let mut renderer = Renderer::new();
                    let mut needs_render = false;

                    loop {
                        let mut wait = Duration::from_secs(0);

                        loop {
                            match if wait.as_micros() == 0 {
                                rx.recv().ok()
                            } else {
                                rx.recv_timeout(wait).ok()
                            } {
                                None => (),
                                Some(Message::Shutdown) => return,
                                Some(Message::Run(mut closure)) => {
                                    closure(&mut renderer);

                                    needs_render = true;
                                }
                            }

                            wait = sync.wait();

                            if wait.as_micros() == 0 {
                                break;
                            }
                        }

                        if needs_render {
                            needs_render = false;

                            sync.tick();
                            renderer.render().unwrap()
                        }
                    }
                }),
            ));
        }
    }
}
