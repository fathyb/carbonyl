use std::ops::Mul;

use crate::gfx::Color;

struct KDNode {
    left: Option<Box<KDNode>>,
    right: Option<Box<KDNode>>,
    normal: Color<f64>,
    middle: (usize, Color<f64>),
}

impl KDNode {
    fn new(colors: &[Color]) {
        let (sum, sum_squared) = colors.iter().fold(
            (Color::black(), Color::black()),
            |(sum, sum_squared), color| (sum + color, sum_squared + color * color),
        );
    }

    fn nearest(&self, color: Color<f64>, mut limit: f64) -> Option<(usize, f64)> {
        let diff = color - self.middle.1;
        let distance = diff.mul(&diff).sum().sqrt();
        let mut result = None;

        if distance < limit {
            limit = distance;
        }

        let dot = diff.mul(self.normal).sum();

        if dot <= 0.0 {
            if let Some(ref left) = self.left {
                if let Some(nearest) = left.nearest(color, limit) {
                    limit = nearest.1;
                    result = Some(nearest);
                }
            }

            if -dot < limit {
                if let Some(ref right) = self.right {
                    if let Some(nearest) = right.nearest(color, limit) {
                        result = Some(nearest);
                    }
                }
            }
        } else {
            if let Some(ref right) = self.right {
                if let Some(nearest) = right.nearest(color, limit) {
                    limit = nearest.1;
                    result = Some(nearest);
                }
            }

            if dot < limit {
                if let Some(ref left) = self.left {
                    if let Some(nearest) = left.nearest(color, limit) {
                        result = Some(nearest);
                    }
                }
            }
        }

        result
    }
}
