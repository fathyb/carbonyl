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
        match self.cursor {
            None => match (key.alt, key.char) {
                (true, 0x14) => NavigationAction::GoBack(),
                (true, 0x13) => NavigationAction::GoForward(),
                _ => NavigationAction::Forward,
            },
            Some(cursor) => {
                if let Some(url) = &mut self.url {
                    // TODO: Unicode
                    match key.char {
                        // Return
                        0x0d => return NavigationAction::GoTo(url.clone()),
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
        let space = size.width as usize - 13;
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
