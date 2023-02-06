use unicode_width::UnicodeWidthStr;

use crate::gfx::Size;

pub struct Navigation {
    url: Option<String>,
    size: Size,
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            url: None,
            size: Size::splat(0),
        }
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = Some(url.to_owned())
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size
    }

    pub fn render(&self, size: Size) -> String {
        let left = "[\u{276e}][\u{276f}][↻][ ";
        let right = " ]";
        let space = size.width as usize - 13;
        let url = match &self.url {
            None => "about:blank".to_owned(),
            Some(url) => url.as_str().chars().take(space).collect(),
        };

        format!(
            "{left}{url}{space}{right}",
            space = " ".repeat(space - url.width())
        )
    }
}
