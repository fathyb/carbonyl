use std::{
    io::{self, Write},
    rc::Rc,
};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    gfx::{Color, Point, Rect, Size},
    input::Key,
    ui::navigation::{Navigation, NavigationAction},
    ui::vimium::{HintTarget, Mode},
    utils::log,
};

use super::{Cell, Grapheme, Painter};

pub struct Renderer {
    nav: Navigation,
    cells: Vec<(Cell, Cell)>,
    painter: Painter,
    size: Size,
    overlay_restore: Vec<(usize, Cell)>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            nav: Navigation::new(),
            cells: Vec::with_capacity(0),
            painter: Painter::new(),
            size: Size::new(0, 0),
            overlay_restore: Vec::new(),
        }
    }

    pub fn enable_true_color(&mut self) {
        self.painter.set_true_color(true)
    }

    pub fn keypress(&mut self, key: &Key, viewport_px_height: i32) -> io::Result<NavigationAction> {
        let action = self.nav.keypress(key, viewport_px_height);

        Ok(action)
    }

    pub fn nav(&self) -> &Navigation {
        &self.nav
    }

    pub fn cells_size(&self) -> Size {
        self.size
    }
    pub fn mouse_up(&mut self, origin: Point) -> io::Result<NavigationAction> {
        let action = self.nav.mouse_up(origin);

        Ok(action)
    }
    pub fn mouse_down(&mut self, origin: Point) -> io::Result<NavigationAction> {
        let action = self.nav.mouse_down(origin);

        Ok(action)
    }
    pub fn mouse_move(&mut self, origin: Point) -> io::Result<NavigationAction> {
        let action = self.nav.mouse_move(origin);

        Ok(action)
    }

    pub fn push_nav(&mut self, url: &str, can_go_back: bool, can_go_forward: bool) {
        self.nav.push(url, can_go_back, can_go_forward)
    }

    pub fn get_size(&self) -> Size {
        self.size
    }

    pub fn set_size(&mut self, size: Size) {
        self.nav.set_size(size);
        self.size = size;
        self.overlay_restore.clear();

        let mut x = 0;
        let mut y = 0;
        let bound = size.width - 1;
        let cells = (size.width + size.width * size.height) as usize;

        self.cells.clear();
        self.cells.resize_with(cells, || {
            let cell = (Cell::new(x, y), Cell::new(x, y));

            if x < bound {
                x += 1;
            } else {
                x = 0;
                y += 1;
            }

            cell
        });
    }

    pub fn render(&mut self) -> io::Result<()> {
        let size = self.size;

        self.restore_overlay();

        for (origin, element) in self.nav.render(size) {
            self.fill_rect(
                Rect::new(origin.x, origin.y, element.text.width() as u32, 1),
                element.background,
            );
            self.draw_text(
                &element.text,
                origin * (2, 1),
                Size::splat(0),
                element.foreground,
            );
        }

        self.render_vimium_overlay();

        self.painter.begin()?;

        for (previous, current) in self.cells.iter_mut() {
            if current == previous {
                continue;
            }

            previous.quadrant = current.quadrant;
            previous.grapheme = current.grapheme.clone();

            self.painter.paint(current)?;
        }

        self.painter.end(self.nav.cursor())?;

        Ok(())
    }

    /// Draw the background from a pixel array encoded in RGBA8888
    pub fn draw_background(&mut self, pixels: &[u8], pixels_size: Size, rect: Rect) {
        let viewport = self.size.cast::<usize>();

        if pixels.len() < viewport.width * viewport.height * 8 * 4 {
            log::debug!(
                "unexpected size, actual: {}, expected: {}",
                pixels.len(),
                viewport.width * viewport.height * 8 * 4
            );
            return;
        }

        let origin = rect.origin.cast::<f32>().max(0.0) / (2.0, 4.0);
        let size = rect.size.cast::<f32>().max(0.0) / (2.0, 4.0);
        let top = (origin.y.floor() as usize).min(viewport.height);
        let left = (origin.x.floor() as usize).min(viewport.width);
        let right = ((origin.x + size.width).ceil() as usize)
            .min(viewport.width)
            .max(left);
        let bottom = ((origin.y + size.height).ceil() as usize)
            .min(viewport.height)
            .max(top);
        let row_length = pixels_size.width as usize;
        let pixel = |x, y| {
            Color::new(
                pixels[((x + y * row_length) * 4 + 2) as usize],
                pixels[((x + y * row_length) * 4 + 1) as usize],
                pixels[((x + y * row_length) * 4 + 0) as usize],
            )
        };
        let pair = |x, y| pixel(x, y).avg_with(pixel(x, y + 1));

        for y in top..bottom {
            let index = (y + 1) * viewport.width;
            let start = index + left;
            let end = index + right;
            let (mut x, y) = (left * 2, y * 4);

            for (_, cell) in &mut self.cells[start..end] {
                cell.quadrant = (
                    pair(x + 0, y + 0),
                    pair(x + 1, y + 0),
                    pair(x + 1, y + 2),
                    pair(x + 0, y + 2),
                );

                x += 2;
            }
        }
    }

    pub fn clear_text(&mut self) {
        for (_, cell) in self.cells.iter_mut() {
            cell.grapheme = None
        }
    }

    pub fn set_title(&self, title: &str) -> io::Result<()> {
        let mut stdout = io::stdout();

        write!(stdout, "\x1b]0;{title}\x07")?;
        write!(stdout, "\x1b]1;{title}\x07")?;
        write!(stdout, "\x1b]2;{title}\x07")?;

        stdout.flush()
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.draw(rect, |cell| {
            cell.grapheme = None;
            cell.quadrant = (color, color, color, color);
        })
    }

    pub fn draw<F>(&mut self, bounds: Rect, mut draw: F)
    where
        F: FnMut(&mut Cell),
    {
        let origin = bounds.origin.cast::<usize>();
        let size = bounds.size.cast::<usize>();
        let viewport_width = self.size.width as usize;
        let top = origin.y;
        let bottom = top + size.height;

        // Iterate over each row
        for y in top..bottom {
            let left = y * viewport_width + origin.x;
            let right = left + size.width;

            for (_, current) in self.cells[left..right].iter_mut() {
                draw(current)
            }
        }
    }

    /// Render some text into the terminal output
    pub fn draw_text(&mut self, string: &str, origin: Point, size: Size, color: Color) {
        // Get an iterator starting at the text origin
        let len = self.cells.len();
        let viewport = &self.size.cast::<usize>();

        if size.width > 2 && size.height > 2 {
            let origin = (origin.cast::<f32>() / (2.0, 4.0) + (0.0, 1.0)).round();
            let size = (size.cast::<f32>() / (2.0, 4.0)).round();
            let left = (origin.x.max(0.0) as usize).min(viewport.width);
            let right = ((origin.x + size.width).max(0.0) as usize).min(viewport.width);
            let top = (origin.y.max(0.0) as usize).min(viewport.height);
            let bottom = ((origin.y + size.height).max(0.0) as usize).min(viewport.height);

            for y in top..bottom {
                let index = y * viewport.width;
                let start = index + left;
                let end = index + right;

                for (_, cell) in self.cells[start..end].iter_mut() {
                    cell.grapheme = None
                }
            }
        } else {
            // Compute the buffer index based on the position
            let index = origin.x / 2 + (origin.y + 1) / 4 * (viewport.width as i32);
            let mut iter = self.cells[len.min(index as usize)..].iter_mut();

            // Get every Unicode grapheme in the input string
            for grapheme in UnicodeSegmentation::graphemes(string, true) {
                let width = grapheme.width();

                for index in 0..width {
                    // Get the next terminal cell at the given position
                    match iter.next() {
                        // Stop if we're at the end of the buffer
                        None => return,
                        // Set the cell to the current grapheme
                        Some((_, cell)) => {
                            let next = Grapheme {
                                // Create a new shared reference to the text
                                color,
                                index,
                                width,
                                // Export the set of unicode code points for this graphene into an UTF-8 string
                                char: grapheme.to_string(),
                            };

                            if match cell.grapheme {
                                None => true,
                                Some(ref previous) => {
                                    previous.color != next.color || previous.char != next.char
                                }
                            } {
                                cell.grapheme = Some(Rc::new(next))
                            }
                        }
                    }
                }
            }
        }
    }

    fn restore_overlay(&mut self) {
        let len = self.cells.len();
        // Overlays can snapshot the same cell twice in one frame (e.g. a find
        // match under the status bar); only the first snapshot holds page
        // content, so restore in reverse to make it win.
        for (idx, original) in std::mem::take(&mut self.overlay_restore).into_iter().rev() {
            if idx < len {
                let (_, cell) = &mut self.cells[idx];
                cell.quadrant = original.quadrant;
                cell.grapheme = original.grapheme;
            }
        }
    }

    fn snapshot_cell(&mut self, idx: usize) {
        if let Some((_, cell)) = self.cells.get(idx) {
            self.overlay_restore.push((
                idx,
                Cell {
                    cursor: cell.cursor,
                    grapheme: cell.grapheme.clone(),
                    quadrant: cell.quadrant,
                },
            ));
        }
    }

    fn render_vimium_overlay(&mut self) {
        let viewport_w = self.size.width as usize;
        let viewport_h = self.size.height as usize;
        if viewport_w == 0 || viewport_h == 0 {
            return;
        }
        let mode = self.nav.vimium_mode();

        if mode == Mode::Hint && self.nav.vimium_hints().is_empty() {
            let targets = self.build_hint_targets(viewport_w, viewport_h);
            if targets.is_empty() {
                self.nav.vimium_exit_hint();
            } else {
                self.nav.vimium_set_hints(targets);
            }
        }
        let mode = self.nav.vimium_mode();

        let status_text = match mode {
            Mode::Find => Some(format!("/{}", self.nav.vimium_find_buf())),
            Mode::Insert => Some("-- INSERT --".to_string()),
            Mode::Hint => Some(format!(
                "hint ({} link{}): {}  [esc to cancel]",
                self.nav.vimium_hints().len(),
                if self.nav.vimium_hints().len() == 1 { "" } else { "s" },
                self.nav.vimium_hint_buf()
            )),
            Mode::Normal => match self.nav.vimium_find_query() {
                Some(q) => Some(format!("/{}", q)),
                None => None,
            },
        };

        let mut matches: Vec<(usize, usize, usize)> = Vec::new();
        let find_query = self
            .nav
            .vimium_find_query()
            .map(|s| s.to_ascii_lowercase());
        let find_cursor = self.nav.vimium_find_cursor();

        if let Some(query) = &find_query {
            if !query.is_empty() {
                matches = self.find_matches(query, viewport_w, viewport_h);
            }
        }

        if !matches.is_empty() {
            let current_idx = find_cursor % matches.len();
            let highlight_bg = Color::new(255, 235, 110);
            let current_bg = Color::new(255, 150, 50);
            for (i, (row, c_start, c_end)) in matches.iter().enumerate() {
                let bg = if i == current_idx {
                    current_bg
                } else {
                    highlight_bg
                };
                let row_off = row * viewport_w;
                for col in *c_start..*c_end {
                    let idx = row_off + col;
                    self.snapshot_cell(idx);
                    if let Some((_, cell)) = self.cells.get_mut(idx) {
                        cell.quadrant = (bg, bg, bg, bg);
                    }
                }
            }
        }

        if mode == Mode::Hint {
            let buf = self.nav.vimium_hint_buf().to_string();
            let hints: Vec<HintTarget> = self
                .nav
                .vimium_hints()
                .iter()
                .filter(|h| h.label.starts_with(&buf))
                .cloned()
                .collect();
            let label_bg = Color::new(255, 220, 60);
            let label_fg = Color::splat(0);
            let typed_fg = Color::new(140, 60, 0);
            for hint in &hints {
                let row = hint.row as usize;
                if row == 0 || row >= viewport_h {
                    continue;
                }
                let row_off = row * viewport_w;
                let label_len = hint.label.chars().count();
                let start_col = hint.col_start as usize;
                for i in 0..label_len {
                    let col = start_col + i;
                    if col >= viewport_w {
                        break;
                    }
                    let idx = row_off + col;
                    self.snapshot_cell(idx);
                    let ch = hint.label.chars().nth(i).unwrap_or(' ');
                    let fg = if i < buf.len() { typed_fg } else { label_fg };
                    if let Some((_, cell)) = self.cells.get_mut(idx) {
                        cell.quadrant = (label_bg, label_bg, label_bg, label_bg);
                        cell.grapheme = Some(Rc::new(Grapheme {
                            char: ch.to_string(),
                            index: 0,
                            width: 1,
                            color: fg,
                        }));
                    }
                }
            }
        }

        // Status bar last so match highlights and hint labels on the bottom
        // row can't paint over it.
        if let Some(text) = &status_text {
            let suffix = if matches.is_empty() && find_query.is_some() {
                " (no matches)".to_string()
            } else if !matches.is_empty() {
                let cur = find_cursor % matches.len();
                format!(" [{}/{}]", cur + 1, matches.len())
            } else {
                String::new()
            };
            let full = format!("{}{}", text, suffix);
            let row = viewport_h.saturating_sub(1).max(1);
            let bg = Color::splat(20);
            let fg = Color::new(255, 255, 255);
            let max_chars = viewport_w.saturating_sub(1);
            let printed: String = full.chars().take(max_chars).collect();
            self.overlay_row(row, viewport_w, bg);
            self.overlay_text(&printed, row, viewport_w, fg);
        }
    }

    fn build_hint_targets(&self, viewport_w: usize, viewport_h: usize) -> Vec<HintTarget> {
        let dominant = self.dominant_text_color(viewport_w, viewport_h);
        // Step 1: walk cells per row, build same-color contiguous text spans.
        // A new span starts on a color change or whitespace boundary.
        let mut spans: Vec<(u32, u32, u32, Color)> = Vec::new();
        for row in 1..=viewport_h {
            let row_off = row * viewport_w;
            if row_off + viewport_w > self.cells.len() {
                break;
            }
            let mut col = 0usize;
            let mut current: Option<(usize, Color)> = None;
            while col < viewport_w {
                let cell = &self.cells[row_off + col].1;
                let info = match &cell.grapheme {
                    Some(g) if g.index == 0 => {
                        let ch = g.char.chars().next().unwrap_or(' ');
                        if ch.is_whitespace() {
                            None
                        } else {
                            Some((g.color, g.width.max(1)))
                        }
                    }
                    Some(_) => Some((Color::black(), 1)),
                    None => None,
                };
                match (current, info) {
                    (Some((start, color)), Some((c, w))) => {
                        if color_distance(color, c) < 15 {
                            col += w;
                        } else {
                            spans.push((row as u32, start as u32, col as u32, color));
                            current = Some((col, c));
                            col += w;
                        }
                    }
                    (Some((start, color)), None) => {
                        spans.push((row as u32, start as u32, col as u32, color));
                        current = None;
                        col += 1;
                    }
                    (None, Some((c, w))) => {
                        current = Some((col, c));
                        col += w;
                    }
                    (None, None) => {
                        col += 1;
                    }
                }
            }
            if let Some((start, color)) = current {
                spans.push((row as u32, start as u32, col as u32, color));
            }
        }

        // Step 2: merge adjacent same-color spans on the same row with a 1-cell
        // gap (single whitespace). Body paragraphs collapse into one wide span;
        // nav menu items separated by multiple spaces stay distinct.
        let mut merged: Vec<(u32, u32, u32, Color)> = Vec::new();
        for span in spans {
            let (row, start, end, color) = span;
            let can_merge = merged.last().map_or(false, |&(r, _, last_end, last_color)| {
                r == row
                    && start.saturating_sub(last_end) <= 1
                    && color_distance(color, last_color) < 15
            });
            if can_merge {
                merged.last_mut().unwrap().2 = end;
            } else {
                merged.push((row, start, end, color));
            }
        }

        // Step 3: keep spans that are either short (likely a button/link/label)
        // or visually distinct from the dominant text color (likely a styled
        // link in body content).
        let max_width = 30u32;
        let filtered: Vec<(u32, u32, u32)> = merged
            .into_iter()
            .filter(|(_, s, e, color)| {
                let width = e.saturating_sub(*s);
                let differs_from_dominant = dominant
                    .map(|d| color_distance(*color, d) >= 60)
                    .unwrap_or(true);
                width >= 1 && (width <= max_width || differs_from_dominant)
            })
            .map(|(r, s, e, _)| (r, s, e))
            .collect();

        let labels = make_hint_labels(filtered.len());
        filtered
            .into_iter()
            .zip(labels.into_iter())
            .map(|((row, col_start, col_end), label)| HintTarget {
                label,
                row,
                col_start,
                col_end,
            })
            .collect()
    }

    fn dominant_text_color(&self, viewport_w: usize, viewport_h: usize) -> Option<Color> {
        let mut buckets: Vec<(Color, usize)> = Vec::new();
        for row in 1..=viewport_h {
            let row_off = row * viewport_w;
            if row_off + viewport_w > self.cells.len() {
                break;
            }
            for col in 0..viewport_w {
                let cell = &self.cells[row_off + col].1;
                if let Some(g) = &cell.grapheme {
                    if g.index != 0 {
                        continue;
                    }
                    if g.char.chars().next().map(|c| c.is_whitespace()).unwrap_or(true) {
                        continue;
                    }
                    let c = g.color;
                    if let Some(slot) = buckets.iter_mut().find(|(b, _)| *b == c) {
                        slot.1 += 1;
                    } else {
                        buckets.push((c, 1));
                    }
                }
            }
        }
        buckets.into_iter().max_by_key(|(_, n)| *n).map(|(c, _)| c)
    }

    fn overlay_row(&mut self, row: usize, viewport_w: usize, bg: Color) {
        let row_off = row * viewport_w;
        for col in 0..viewport_w {
            let idx = row_off + col;
            self.snapshot_cell(idx);
            if let Some((_, cell)) = self.cells.get_mut(idx) {
                cell.quadrant = (bg, bg, bg, bg);
                cell.grapheme = None;
            }
        }
    }

    fn overlay_text(&mut self, text: &str, row: usize, viewport_w: usize, fg: Color) {
        let row_off = row * viewport_w;
        let mut col = 0usize;
        for grapheme in UnicodeSegmentation::graphemes(text, true) {
            let width = grapheme.width().max(1);
            if col >= viewport_w {
                break;
            }
            for i in 0..width {
                if col + i >= viewport_w {
                    return;
                }
                let idx = row_off + col + i;
                if let Some((_, cell)) = self.cells.get_mut(idx) {
                    cell.grapheme = Some(Rc::new(Grapheme {
                        char: grapheme.to_string(),
                        index: i,
                        width,
                        color: fg,
                    }));
                }
            }
            col += width;
        }
    }

    fn find_matches(
        &self,
        query: &str,
        viewport_w: usize,
        viewport_h: usize,
    ) -> Vec<(usize, usize, usize)> {
        let mut out = Vec::new();
        let max_row = viewport_h;
        for row in 1..=max_row {
            let row_off = row * viewport_w;
            if row_off + viewport_w > self.cells.len() {
                break;
            }
            let mut row_text = String::new();
            let mut byte_to_col: Vec<(usize, usize)> = Vec::new();
            let mut col = 0usize;
            while col < viewport_w {
                let cell = &self.cells[row_off + col].1;
                if let Some(g) = &cell.grapheme {
                    if g.index == 0 {
                        byte_to_col.push((row_text.len(), col));
                        row_text.push_str(&g.char);
                        col += g.width.max(1);
                        continue;
                    }
                }
                byte_to_col.push((row_text.len(), col));
                row_text.push(' ');
                col += 1;
            }
            let row_lower = row_text.to_ascii_lowercase();
            let mut start_byte = 0usize;
            while start_byte <= row_lower.len() {
                let slice = &row_lower[start_byte..];
                let Some(rel) = slice.find(query) else { break };
                let abs_byte = start_byte + rel;
                let end_byte = abs_byte + query.len();
                let col_start = byte_to_col
                    .iter()
                    .rev()
                    .find(|(b, _)| *b <= abs_byte)
                    .map(|(_, c)| *c)
                    .unwrap_or(0);
                let col_end = byte_to_col
                    .iter()
                    .find(|(b, _)| *b >= end_byte)
                    .map(|(_, c)| *c)
                    .unwrap_or(viewport_w);
                let col_end = col_end.max(col_start + 1);
                out.push((row, col_start, col_end));
                start_byte = abs_byte + query.len().max(1);
            }
        }
        out
    }
}

fn color_distance(a: Color, b: Color) -> u32 {
    let dr = (a.r as i32 - b.r as i32).unsigned_abs();
    let dg = (a.g as i32 - b.g as i32).unsigned_abs();
    let db = (a.b as i32 - b.b as i32).unsigned_abs();
    dr + dg + db
}

fn make_hint_labels(n: usize) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }
    if n <= 26 {
        return (0..n)
            .map(|i| ((b'a' + i as u8) as char).to_string())
            .collect();
    }
    if n <= 26 * 26 {
        return (0..n)
            .map(|i| {
                let hi = i / 26;
                let lo = i % 26;
                let mut s = String::new();
                s.push((b'a' + hi as u8) as char);
                s.push((b'a' + lo as u8) as char);
                s
            })
            .collect();
    }
    (0..n)
        .map(|i| {
            let h = i / (26 * 26);
            let m = (i / 26) % 26;
            let l = i % 26;
            let mut s = String::new();
            s.push((b'a' + h as u8) as char);
            s.push((b'a' + m as u8) as char);
            s.push((b'a' + l as u8) as char);
            s
        })
        .collect()
}
