use std::time::{Duration, Instant};

/// A utility to synchronize rendering with a given FPS
pub struct FrameSync {
    render_start: Option<Instant>,
    frame_duration: Duration,
}

impl FrameSync {
    pub fn new(fps: f32) -> Self {
        Self {
            render_start: None,
            frame_duration: Duration::from_micros((1_000_000.0 / fps) as u64),
        }
    }

    /// Mark the beginning of the render
    pub fn start(&mut self) {
        self.render_start = Some(Instant::now());
    }

    /// Get a deadline until the next frame
    pub fn deadline(&self) -> Instant {
        match self.render_start {
            // We never rendered yet, render now!
            None => Instant::now(),
            // Else we should render `frame_duration` after the last render start.
            // If we render at 60 FPS, this should be 16ms after the render start.
            // If the render takes more than the frame duration, this will always
            // return a deadline in a the past, making render happen immediately.
            Some(render_start) => render_start + self.frame_duration,
        }
    }
}
