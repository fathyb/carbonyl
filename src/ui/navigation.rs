use std::env;

use unicode_width::UnicodeWidthStr;

use crate::{
    gfx::{Color, Point, Size},
    input::Key,
    utils::log,
};

pub enum NavigationAction {
    Ignore,
    Forward,
    GoTo(String),
    GoBack(),
    GoForward(),
    Refresh(),
}

#[derive(Debug)]
pub struct NavigationElement {
    pub text: String,
    pub background: Color,
    pub foreground: Color,
}

pub struct Navigation {
    url: Option<String>,
    size: Size,
    cursor: Option<usize>,
    can_go_back: bool,
    can_go_forward: bool,
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            url: None,
            size: (0, 0).into(),
            cursor: None,
            can_go_back: false,
            can_go_forward: false,
        }
    }

    pub fn cursor(&self) -> Option<Point> {
        Some((11 + self.cursor? as i32, 0).into())
    }

    pub fn keypress(&mut self, key: &Key) -> NavigationAction {
        let modifier_key = match env::consts::OS {
            "macos" => key.modifiers.meta,
            _ => key.modifiers.alt,
        };

        match self.cursor {
            None => match (modifier_key, key.char) {
                (true, 0x14) => NavigationAction::GoBack(),
                (true, 0x13) => NavigationAction::GoForward(),
                _ => NavigationAction::Forward,
            },
            Some(cursor) => {
                if let Some(url) = &mut self.url {
                    // TODO: Unicode
                    match key.char {
                        // Return (CR or LF)
                        0x0d | 0x0a => return NavigationAction::GoTo(resolve_url(url)),
                        // Up
                        0x11 => self.cursor = Some(0),
                        // Down
                        0x12 => self.cursor = Some(url.width()),
                        // Right
                        0x13 => self.cursor = Some((cursor + 1).min(url.width())),
                        // Left
                        0x14 => self.cursor = Some(if cursor > 0 { cursor - 1 } else { 0 }),
                        // Backspace
                        0x7f => {
                            if cursor > 0 {
                                url.remove(cursor - 1);

                                self.cursor = Some(cursor - 1);
                            }
                        }
                        key => {
                            url.insert(cursor, key as char);

                            self.cursor = Some((cursor + 1).min(url.width()))
                        }
                    }

                    NavigationAction::Ignore
                } else {
                    NavigationAction::Forward
                }
            }
        }
    }

    pub fn display_url(&self) -> &str {
        match &self.url {
            None => "about:blank",
            Some(url) => url,
        }
    }

    pub fn url_size(&self) -> usize {
        self.display_url().width()
    }

    pub fn mouse_up(&mut self, origin: Point) -> NavigationAction {
        if origin.y != 0 {
            self.cursor = None;

            NavigationAction::Forward
        } else {
            NavigationAction::Ignore
        }
    }
    pub fn mouse_down(&mut self, origin: Point) -> NavigationAction {
        if origin.y != 0 {
            self.cursor = None;

            return NavigationAction::Forward;
        }

        self.cursor = None;

        return match origin.x {
            0..=2 => NavigationAction::GoBack(),
            3..=5 => NavigationAction::GoForward(),
            6..=8 => NavigationAction::Refresh(),
            11.. => {
                self.cursor = Some(self.url_size().min(origin.x as usize - 11));

                log::debug!("setting cursor to {:?}", self.cursor);

                NavigationAction::Ignore
            }
            _ => NavigationAction::Ignore,
        };
    }
    pub fn mouse_move(&mut self, _origin: Point) -> NavigationAction {
        NavigationAction::Forward
    }

    pub fn push(&mut self, url: &str, can_go_back: bool, can_go_forward: bool) {
        if match (self.cursor, &self.url) {
            (None, _) => false,
            (_, None) => true,
            (_, Some(current)) => current != url,
        } {
            self.cursor = Some(url.len())
        }

        self.url = Some(url.to_owned());
        self.can_go_back = can_go_back;
        self.can_go_forward = can_go_forward;
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size
    }

    pub fn render_btn(&self, icon: &str, enabled: bool) -> [NavigationElement; 3] {
        let background = Color::splat(255);
        let foreground = Color::splat(0);

        [
            NavigationElement {
                text: "[".to_owned(),
                background,
                foreground,
            },
            NavigationElement {
                text: icon.to_owned(),
                background,
                foreground: if enabled {
                    foreground
                } else {
                    Color::splat(200)
                },
            },
            NavigationElement {
                text: "]".to_owned(),
                background,
                foreground,
            },
        ]
    }

    pub fn render(&self, size: Size) -> Vec<(Point, NavigationElement)> {
        let ui_elements = 13;
        let space = if size.width >= ui_elements {
            (size.width - ui_elements) as usize
        } else {
            0
        };
        let url: String = self.display_url().chars().take(space).collect();
        let width = url.width();
        let padded = format!(" {}{} ", url, " ".repeat(space - width));
        let mut elements = Vec::new();
        let mut point = Point::splat(0);

        for list in [
            self.render_btn("\u{276e}", self.can_go_back),
            self.render_btn("\u{276f}", self.can_go_forward),
            self.render_btn("↻", true),
            self.render_btn(&padded, true),
        ] {
            for element in list {
                let width = element.text.width() as i32;

                elements.push((point.clone(), element));

                point = point + (width, 0);
            }
        }

        elements
    }
}

/// Turn what the user typed in the address bar into something Chromium can load,
/// the way a browser omnibox does: keep anything that already has a scheme,
/// assume `https://` for a bare host like `youtube.com`, and send everything
/// else to a search engine. Without this, typing `youtube.com` and pressing
/// Return does nothing, because the raw string isn't a valid URL.
fn resolve_url(input: &str) -> String {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return "about:blank".to_owned();
    }

    // Already a full URL or a special scheme: load it verbatim.
    if trimmed.contains("://")
        || trimmed.starts_with("about:")
        || trimmed.starts_with("data:")
        || trimmed.starts_with("file:")
    {
        return trimmed.to_owned();
    }

    // A bare host such as `youtube.com`, `localhost` or `localhost:3000`.
    let host = trimmed.split(['/', '?', '#']).next().unwrap_or(trimmed);
    let host_only = host.split(':').next().unwrap_or(host);
    let looks_like_host =
        !trimmed.contains(' ') && (host_only.contains('.') || host_only == "localhost");

    if looks_like_host {
        format!("https://{trimmed}")
    } else {
        format!("https://duckduckgo.com/?q={}", percent_encode(trimmed))
    }
}

/// Percent-encode a search query for use in a URL.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::resolve_url;

    #[test]
    fn keeps_full_urls() {
        assert_eq!(resolve_url("https://youtube.com"), "https://youtube.com");
        assert_eq!(resolve_url("http://example.com/x"), "http://example.com/x");
        assert_eq!(resolve_url("about:blank"), "about:blank");
        assert_eq!(resolve_url("file:///etc/hosts"), "file:///etc/hosts");
    }

    #[test]
    fn assumes_https_for_bare_hosts() {
        assert_eq!(resolve_url("youtube.com"), "https://youtube.com");
        assert_eq!(resolve_url("youtube.com/watch?v=1"), "https://youtube.com/watch?v=1");
        assert_eq!(resolve_url("localhost"), "https://localhost");
        assert_eq!(resolve_url("localhost:3000"), "https://localhost:3000");
        assert_eq!(resolve_url("  arxiv.org  "), "https://arxiv.org");
    }

    #[test]
    fn searches_plain_text() {
        assert_eq!(
            resolve_url("hello world"),
            "https://duckduckgo.com/?q=hello+world"
        );
        assert_eq!(resolve_url("rust"), "https://duckduckgo.com/?q=rust");
    }

    #[test]
    fn empty_is_blank() {
        assert_eq!(resolve_url("   "), "about:blank");
    }
}
