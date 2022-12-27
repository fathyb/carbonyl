use std::os::unix::prelude::AsRawFd;
use std::{fs, io};

/// Setup the input stream to operate in raw mode.
/// Allows for reading characters without waiting for the return key to be pressed.
pub fn setup() -> io::Result<()> {
    unsafe {
        let tty;

        let fd = if libc::isatty(libc::STDIN_FILENO) == 1 {
            libc::STDIN_FILENO
        } else {
            // Use /dev/tty in the input stream is not a terminal.
            // Happens if something is piped to stdin.
            tty = fs::File::open("/dev/tty")?;

            tty.as_raw_fd()
        };

        let mut ptr = core::mem::MaybeUninit::uninit();

        // Load the terminal parameters
        if libc::tcgetattr(fd, ptr.as_mut_ptr()) == 0 {
            let mut termios = ptr.assume_init();
            let c_oflag = termios.c_oflag;

            // Set the terminal to raw mode
            libc::cfmakeraw(&mut termios);
            // Restore output flags, ensures carriage returns are consistent
            termios.c_oflag = c_oflag;

            // Save the terminal parameters
            if libc::tcsetattr(fd, libc::TCSANOW, &termios) == 0 {
                return Ok(());
            }
        }
    }

    Err(io::Error::last_os_error())
}
