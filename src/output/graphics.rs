use std::io::{self, Write};

use crate::gfx::Size;

/// Renders the browser framebuffer to the terminal using the kitty graphics
/// protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
///
/// The default renderer downsamples every 2x4 pixel block to a single terminal
/// cell and quantizes it to two colors (see `quad.rs`), which is what makes the
/// output look blocky. On terminals that implement the kitty graphics protocol
/// we can instead hand the raw framebuffer to the terminal, which composites it
/// at full pixel resolution — crisp text and images, straight from Blink.
///
/// This requires the page (including text) to be present in the framebuffer,
/// so graphics mode implies bitmap mode (see `Bridge::BitmapMode`).
pub struct KittyGraphics {
    /// Latest framebuffer, packed as RGB (3 bytes per pixel).
    rgb: Vec<u8>,
    /// Size of the framebuffer in pixels.
    pixels: Size,
    /// Whether the framebuffer changed since the last `encode`.
    dirty: bool,
    /// Rotating counter for unique temp-file names.
    counter: u64,
    /// Recently written temp files, reaped once the terminal has read them.
    recent: std::collections::VecDeque<String>,
}

impl KittyGraphics {
    pub fn new() -> Self {
        KittyGraphics {
            rgb: Vec::new(),
            pixels: Size::new(0, 0),
            dirty: false,
            counter: 0,
            recent: std::collections::VecDeque::new(),
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    /// Store a new framebuffer. `pixels` is BGRA8888, as handed to us by
    /// Chromium in `Renderer::draw_background`.
    pub fn store(&mut self, pixels: &[u8], size: Size) {
        let count = (size.width as usize) * (size.height as usize);

        if count == 0 || pixels.len() < count * 4 {
            return;
        }

        self.rgb.clear();
        self.rgb.reserve(count * 3);

        for i in 0..count {
            // Source is BGRA, kitty's RGB format (f=24) wants R, G, B.
            self.rgb.push(pixels[i * 4 + 2]);
            self.rgb.push(pixels[i * 4 + 1]);
            self.rgb.push(pixels[i * 4 + 0]);
        }

        self.pixels = size;
        self.dirty = true;
    }

    /// Tell the terminal to forget our image and its data. Used on resize so a
    /// stale placement isn't left scaled to the wrong size. Scoped to image
    /// id 1 (`d=I,i=1`) so we never touch images other programs may have drawn.
    pub fn reset() -> &'static [u8] {
        b"\x1b_Ga=d,d=I,i=1\x1b\\"
    }

    /// Encode the escape codes that draw the framebuffer into a `cols`x`rows`
    /// cell box anchored at the 1-based terminal cell (`col`, `row`). The cursor
    /// is saved and restored so the navigation caret is left untouched.
    ///
    /// The framebuffer is handed to the terminal through a temporary file
    /// (`t=t`) rather than base64'd inline: a single page can be several
    /// megabytes, and pushing that through the pty every frame is the main
    /// source of latency. The terminal reads and deletes the file itself.
    pub fn encode(&mut self, col: u32, row: u32, cols: u32, rows: u32) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();

        self.dirty = false;

        if self.pixels.width == 0 || self.pixels.height == 0 {
            return Ok(out);
        }

        self.counter = self.counter.wrapping_add(1);
        let dir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
        // The name must contain "tty-graphics-protocol" for kitty to delete the
        // file itself after reading it.
        let path = format!(
            "{}/carbonyl-tty-graphics-protocol-{}-{}.rgb",
            dir.trim_end_matches('/'),
            std::process::id(),
            self.counter
        );

        std::fs::write(&path, &self.rgb)?;

        // Reap files from earlier frames in case the terminal didn't (e.g. it
        // isn't kitty, or doesn't honor t=t deletion). By now they've been read.
        self.recent.push_back(path.clone());
        while self.recent.len() > 2 {
            if let Some(old) = self.recent.pop_front() {
                let _ = std::fs::remove_file(old);
            }
        }

        // Save cursor, move to the content origin.
        write!(out, "\x1b7\x1b[{};{}H", row, col)?;

        // a=T  transmit and display
        // f=24 raw RGB
        // t=t  payload is the path to a temporary file the terminal deletes
        // s,v  source size in pixels
        // c,r  number of cells to scale the image into
        // i=1  image id (reused each frame, so the terminal replaces the data)
        // p=1  placement id (reused, so the terminal replaces the placement)
        // q=2  suppress the terminal's success/error replies
        // C=1  do not move the cursor when placing the image
        write!(
            out,
            "\x1b_Ga=T,f=24,t=t,s={},v={},c={},r={},i=1,p=1,q=2,C=1;{}\x1b\\",
            self.pixels.width,
            self.pixels.height,
            cols,
            rows,
            base64(path.as_bytes())
        )?;

        // Restore the cursor.
        write!(out, "\x1b8")?;

        Ok(out)
    }
}

/// Standard base64 (RFC 4648) encoder. Kept inline to avoid pulling in a crate.
fn base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let n = ((chunk[0] as u32) << 16)
            | ((*chunk.get(1).unwrap_or(&0) as u32) << 8)
            | (*chunk.get(2).unwrap_or(&0) as u32);

        out.push(ALPHABET[((n >> 18) & 63) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 63) as usize] as char
        } else {
            '='
        });
    }

    out
}
