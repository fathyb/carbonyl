use super::Vector2;
use crate::impl_vector_overload;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Size<T: Copy = u32> {
    pub width: T,
    pub height: T,
}

impl_vector_overload!(Size width height);
