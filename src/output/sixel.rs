use crate::gfx::Size;
use sixel_bytes::{self, DiffusionMethod, PixelFormat};

#[derive(Clone, Debug)]
pub struct Frame {
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    Encode(sixel_bytes::SixelError),
    InvalidSize(Size<u32>),
}

impl From<sixel_bytes::SixelError> for Error {
    fn from(value: sixel_bytes::SixelError) -> Self {
        Self::Encode(value)
    }
}

impl Frame {
    fn encode_rgba(pixels: &[u8], size: Size<u32>, method: DiffusionMethod) -> Result<Self, Error> {
        if size.width == 0 || size.height == 0 {
            return Err(Error::InvalidSize(size));
        }

        let bytes = sixel_bytes::sixel_string(
            pixels,
            size.width as i32,
            size.height as i32,
            PixelFormat::BGRA8888,
            method,
        )
        .map_err(Error::from)?
        .into_bytes();

        Ok(Self { bytes })
    }

    pub fn from_viewport(
        pixels: &[u8],
        size: Size<u32>,
        method: DiffusionMethod,
    ) -> Result<Self, Error> {
        if size.width == 0 || size.height == 0 {
            return Err(Error::InvalidSize(size));
        }

        Self::encode_rgba(pixels, size, method)
    }
}
