use std::ops::Range;

use super::{TextDirection, TextGlyph};

/// 一条形状化硬行的字形视图；UI 折行与溢出策略由后续布局层处理。
#[derive(Clone, Debug, PartialEq)]
pub struct TextShapeRun {
    pub source_range: Range<usize>,
    pub direction: TextDirection,
    pub glyphs: Vec<TextGlyph>,
}
