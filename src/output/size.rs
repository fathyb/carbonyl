use core::mem::MaybeUninit;
use std::io;

use crate::gfx::Size;

pub fn size() -> io::Result<Size> {
    let mut ptr = MaybeUninit::<libc::winsize>::uninit();

    unsafe {
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ptr.as_mut_ptr()) == 0 {
            let mut size = ptr.assume_init();

            if size.ws_col == 0 || size.ws_row == 0 {
                eprintln!(
                    "TIOCGWINSZ returned an empty size ({}x{}), defaulting to 80x24",
                    size.ws_col, size.ws_row
                );

                size.ws_col = 80;
                size.ws_row = 24;
            }

            Ok(Size::new(size.ws_col as u32, size.ws_row as u32))
        } else {
            Err(io::Error::last_os_error())
        }
    }
}
