use crate::gfx::Color;

impl Color {
    pub fn to_xterm(&self) -> u8 {
        if self.max_val() - self.min_val() < 5 {
            match self.r {
                r if r < 4 => 16,
                r if r < 8 => 232,
                r if r > 246 => 231,
                r if r > 238 => 255,
                r => 232 + (r - 8) / 10,
            }
        } else {
            let scale = 5.0 / 200.0;

            (16.0
                + self
                    .cast::<f32>()
                    .mul_add(scale, -55.0 * scale)
                    .max(0.0)
                    .round()
                    .dot((36.0, 6.0, 1.0))) as u8
        }
    }
}
