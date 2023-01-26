use std::rc::Rc;

use crate::gfx::{Color, Point};

#[derive(Clone, PartialEq)]
pub struct Grapheme {
    /// Unicode character in UTF-8, might contain multiple code points (Emoji, CJK).
    pub char: String,
    pub index: usize,
    pub width: usize,
    pub color: Color,
}

/// Terminal cell with `height = width * 2`
#[derive(PartialEq)]
pub struct Cell {
    /// Top pixel color value
    pub top: Color,
    /// Bottom pixel color value
    pub bottom: Color,
    pub cursor: Point<u32>,
    /// Text grapheme if any
    pub grapheme: Option<Rc<Grapheme>>,
}

impl Cell {
    pub fn new(x: u32, y: u32) -> Cell {
        Cell {
            top: Color::black(),
            bottom: Color::black(),
            cursor: Point::new(x, y),
            grapheme: None,
        }
    }
}
