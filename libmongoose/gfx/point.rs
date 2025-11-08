use super::{Rect, Vector2};
use crate::impl_vector_overload;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point<T: Copy = i32> {
    pub x: T,
    pub y: T,
}

impl Point {
    pub fn inside(&self, rect: Rect) -> bool {
        self.x >= rect.origin.x
            && self.y >= rect.origin.y
            && self.x < rect.origin.x + rect.size.width as i32
            && self.y < rect.origin.y + rect.size.height as i32
    }
}

impl_vector_overload!(Point x y);
