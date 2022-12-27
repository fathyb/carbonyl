use crate::gfx::Color;

impl Color {
    pub fn to_xterm(&self) -> u8 {
        if self.r == self.g && self.g == self.b && self.r > 4 && self.r < 239 {
            232 + (self.r - 8) / 10
        } else {
            (16.0
                + self
                    .cast::<f32>()
                    .mul_add(5.0 / 200.0, -(55.0 * (5.0 / 200.0)))
                    .max(0.0)
                    .round()
                    .dot((36.0, 6.0, 1.0))) as u8
        }
    }
}
