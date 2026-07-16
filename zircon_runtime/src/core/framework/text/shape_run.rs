use std::ops::Range;

use super::{TextDirection, TextGlyph};

#[derive(Clone, Debug, PartialEq)]
pub struct TextShapeRun {
    pub source_range: Range<usize>,
    pub direction: TextDirection,
    pub glyphs: Vec<TextGlyph>,
}
