use super::{Point, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect<P: Copy = i32, S: Copy = u32> {
    pub origin: Point<P>,
    pub size: Size<S>,
}

impl<P: Copy, S: Copy> Rect<P, S> {
    pub fn new(x: P, y: P, width: S, height: S) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }
}
