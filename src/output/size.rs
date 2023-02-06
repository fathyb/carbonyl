use core::mem::MaybeUninit;
use std::{io, str::FromStr};

use crate::{gfx::Size, utils::log};

pub fn size() -> io::Result<Size> {
    let mut ptr = MaybeUninit::<libc::winsize>::uninit();
    let mut size = unsafe {
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ptr.as_mut_ptr()) == 0 {
            Some(ptr.assume_init())
        } else {
            None
        }
    }
    .ok_or_else(io::Error::last_os_error)?;

    if size.ws_col == 0 || size.ws_row == 0 {
        let cols = parse_var("COLUMNS").unwrap_or(80);
        let rows = parse_var("LINES").unwrap_or(24);

        log::warning!(
            "TIOCGWINSZ returned an empty size ({}x{}), defaulting to {}x{}",
            size.ws_col,
            size.ws_row,
            cols,
            rows
        );

        size.ws_col = cols;
        size.ws_row = rows;
    }

    Ok(Size::new(
        size.ws_col,
        // Keep some space for the UI
        if size.ws_row > 0 {
            size.ws_row - 1
        } else {
            size.ws_row
        },
    )
    .cast())
}

fn parse_var<T: FromStr>(var: &str) -> Option<T> {
    std::env::var(var).ok()?.parse().ok()
}
