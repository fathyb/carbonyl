use core::mem::MaybeUninit;
use std::io;

use crate::gfx::Size;

pub fn size() -> io::Result<Size> {
    let mut ptr = MaybeUninit::<libc::winsize>::uninit();

    unsafe {
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ptr.as_mut_ptr()) == 0 {
            let size = ptr.assume_init();

            Ok(Size::new(size.ws_col as u32, size.ws_row as u32))
        } else {
            Err(io::Error::last_os_error())
        }
    }
}
