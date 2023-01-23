use super::{Point, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect<P: Copy = i32, S: Copy = u32> {
    pub origin: Point<P>,
    pub size: Size<S>,
}
