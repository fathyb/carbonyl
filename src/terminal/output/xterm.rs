use crate::gfx::Color;

impl Color {
    pub fn to_xterm(&self) -> u8 {
        if self.max_val() - self.min_val() < 8 {
            match self.r {
                0..=4 => 16,
                5..=8 => 232,
                238..=246 => 255,
                247..=255 => 231,
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
