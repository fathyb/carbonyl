use super::Vector3;
use crate::impl_vector_overload;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color<T: Copy = u8> {
    pub r: T,
    pub g: T,
    pub b: T,
}

impl Color {
    pub fn from_iter<'a, T>(iter: &mut T) -> Option<Color>
    where
        T: Iterator<Item = &'a u8>,
    {
        let (b, g, r, _) = (iter.next(), iter.next(), iter.next(), iter.next());

        Some(Color::<u8>::new(*r?, *g?, *b?))
    }

    pub fn black() -> Color {
        Color::<u8>::new(0, 0, 0)
    }
}

impl_vector_overload!(Color r g b);
