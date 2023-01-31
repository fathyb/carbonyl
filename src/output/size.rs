use core::mem::MaybeUninit;
use std::io;

use crate::gfx::Size;

pub fn size() -> io::Result<Size> {
    let mut ptr = MaybeUninit::<libc::winsize>::uninit();

    unsafe {
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ptr.as_mut_ptr()) == 0 {
            let size = ptr.assume_init();

            Ok({
                if size.ws_col == 0 || !size.ws_row == 0 {
                    eprintln!("TIOCGWINSZ returned an empty size, defaulting to 80x24");

                    Size::new(80, 24)
                } else {
                    Size::new(size.ws_col as u32, size.ws_row as u32)
                }
            })
        } else {
            Err(io::Error::last_os_error())
        }
    }
}
