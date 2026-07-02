use crate::cli::CommandLine;
use crate::input::Key;

use super::navigation::NavigationAction;

const SCROLL_TO_EDGE_PX: i32 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Find,
    Hint,
}

#[derive(Debug, Clone)]
pub struct HintTarget {
    pub label: String,
    pub row: u32,
    pub col_start: u32,
    pub col_end: u32,
}

impl HintTarget {
    pub fn click_col(&self) -> u32 {
        (self.col_start + self.col_end.saturating_sub(1)) / 2
    }
    pub fn click_row(&self) -> u32 {
        self.row
    }
}

pub struct Vimium {
    mode: Mode,
    pending_g: bool,
    find_buf: String,
    find_query: Option<String>,
    find_cursor: usize,
    hints: Vec<HintTarget>,
    hint_buf: String,
    // Off by default: every key forwards to the page unchanged, like stock
    // carbonyl. `--vim` opts in, mirroring how `--graphics` gates the kitty
    // renderer, so pages with their own keymap keep the full key stream
    // unless the user asks for vimium.
    disabled: bool,
}

impl Vimium {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            pending_g: false,
            find_buf: String::new(),
            find_query: None,
            find_cursor: 0,
            hints: Vec::new(),
            hint_buf: String::new(),
            disabled: !CommandLine::parse().vim,
        }
    }

    pub fn hints(&self) -> &[HintTarget] {
        &self.hints
    }

    pub fn hint_buf(&self) -> &str {
        &self.hint_buf
    }

    pub fn set_hints(&mut self, hints: Vec<HintTarget>) {
        self.hints = hints;
    }

    pub fn exit_hint(&mut self) {
        self.hints.clear();
        self.hint_buf.clear();
        self.mode = Mode::Normal;
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn find_buf(&self) -> &str {
        &self.find_buf
    }

    pub fn find_query(&self) -> Option<&str> {
        self.find_query.as_deref()
    }

    pub fn find_cursor(&self) -> usize {
        self.find_cursor
    }

    pub fn handle(&mut self, key: &Key, viewport_px_height: i32) -> NavigationAction {
        if self.disabled {
            return NavigationAction::Forward;
        }

        let line_step = (viewport_px_height / 16).max(40);
        let half_page = (viewport_px_height / 2).max(line_step);

        match self.mode {
            Mode::Insert => self.handle_insert(key),
            Mode::Find => self.handle_find(key),
            Mode::Hint => self.handle_hint(key),
            Mode::Normal => self.handle_normal(key, line_step, half_page),
        }
    }

    fn handle_hint(&mut self, key: &Key) -> NavigationAction {
        match key.char {
            0x1b => {
                self.exit_hint();
                NavigationAction::Ignore
            }
            0x7f | 0x08 => {
                self.hint_buf.pop();
                NavigationAction::Ignore
            }
            c if c.is_ascii_alphabetic() => {
                self.hint_buf.push((c as char).to_ascii_lowercase());
                let buf = self.hint_buf.as_str();
                let mut last_match: Option<(u32, u32)> = None;
                let mut count = 0usize;
                for h in &self.hints {
                    if h.label.starts_with(buf) {
                        count += 1;
                        last_match = Some((h.click_col(), h.click_row()));
                        if count > 1 {
                            break;
                        }
                    }
                }
                if count == 1 {
                    let (col, row) = last_match.unwrap();
                    self.exit_hint();
                    return NavigationAction::Click(col, row);
                }
                if count == 0 {
                    self.exit_hint();
                }
                NavigationAction::Ignore
            }
            _ => NavigationAction::Ignore,
        }
    }

    fn handle_insert(&mut self, key: &Key) -> NavigationAction {
        if key.char == 0x1b {
            self.mode = Mode::Normal;
            return NavigationAction::Ignore;
        }
        NavigationAction::Forward
    }

    fn handle_find(&mut self, key: &Key) -> NavigationAction {
        match key.char {
            0x1b => {
                self.mode = Mode::Normal;
                self.find_buf.clear();
                NavigationAction::Ignore
            }
            0x0d => {
                self.find_query = if self.find_buf.is_empty() {
                    None
                } else {
                    Some(self.find_buf.clone())
                };
                self.find_cursor = 0;
                self.find_buf.clear();
                self.mode = Mode::Normal;
                NavigationAction::Ignore
            }
            0x7f | 0x08 => {
                if self.find_buf.pop().is_none() {
                    self.mode = Mode::Normal;
                }
                NavigationAction::Ignore
            }
            c if (0x20..0x7f).contains(&c) => {
                self.find_buf.push(c as char);
                NavigationAction::Ignore
            }
            _ => NavigationAction::Ignore,
        }
    }

    fn handle_normal(&mut self, key: &Key, line_step: i32, half_page: i32) -> NavigationAction {
        if self.pending_g {
            self.pending_g = false;
            return match key.char {
                b'g' => NavigationAction::Scroll(SCROLL_TO_EDGE_PX),
                _ => NavigationAction::Ignore,
            };
        }

        match key.char {
            0x1b => {
                self.pending_g = false;
                self.find_query = None;
                self.find_cursor = 0;
                NavigationAction::Ignore
            }
            b'i' => {
                self.mode = Mode::Insert;
                NavigationAction::Ignore
            }
            b'/' => {
                self.mode = Mode::Find;
                self.find_buf.clear();
                NavigationAction::Ignore
            }
            b'h' => NavigationAction::KeyPress(0x14),
            b'l' => NavigationAction::KeyPress(0x13),
            b'j' => NavigationAction::Scroll(-line_step),
            b'k' => NavigationAction::Scroll(line_step),
            b'd' => NavigationAction::Scroll(-half_page),
            b'u' => NavigationAction::Scroll(half_page),
            b'g' => {
                self.pending_g = true;
                NavigationAction::Ignore
            }
            b'G' => NavigationAction::Scroll(-SCROLL_TO_EDGE_PX),
            b'r' => NavigationAction::Refresh(),
            b'H' => NavigationAction::GoBack(),
            b'L' => NavigationAction::GoForward(),
            b'n' => {
                if self.find_query.is_some() {
                    self.find_cursor = self.find_cursor.wrapping_add(1);
                }
                NavigationAction::Ignore
            }
            b'N' => {
                if self.find_query.is_some() {
                    self.find_cursor = self.find_cursor.wrapping_sub(1);
                }
                NavigationAction::Ignore
            }
            b'f' => {
                self.mode = Mode::Hint;
                self.hint_buf.clear();
                self.hints.clear();
                NavigationAction::Ignore
            }
            // Arrows (0x11-0x14), Enter, Tab, backspace. Swallowing these
            // would kill basic page interaction (focus cycling, link
            // activation), so pass them through.
            c if c < 0x20 || c == 0x7f => NavigationAction::Forward,
            _ => NavigationAction::Ignore,
        }
    }
}
