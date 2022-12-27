use super::Vector2;
use crate::impl_vector_overload;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point<T: Copy = i32> {
    pub x: T,
    pub y: T,
}

impl_vector_overload!(Point x y);
