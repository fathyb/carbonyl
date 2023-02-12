use std::time::{Duration, Instant};

pub struct FrameSync {
    last_render: Option<Instant>,
    frame_duration: Duration,
}

impl FrameSync {
    pub fn new(fps: f32) -> Self {
        Self {
            last_render: None,
            frame_duration: Duration::from_micros((1_000_000.0 / fps) as u64),
        }
    }

    pub fn tick(&mut self) {
        self.last_render = Some(Instant::now());
    }

    pub fn wait(&mut self) -> Duration {
        if let Some(last_render) = self.last_render {
            let elapsed = Instant::now() - last_render;

            if elapsed < self.frame_duration {
                return self.frame_duration - elapsed;
            }
        }

        Duration::from_secs(0)
    }
}
