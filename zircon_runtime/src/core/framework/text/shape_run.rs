use std::ops::Range;

use super::{TextDirection, TextGlyph};

/// 一条形状化硬行的字形视图；UI 折行与溢出策略由后续布局层处理。
///
/// # 坐标与生命周期
/// 条目拥有字形向量和范围数值，不借用源文本；范围使用对应请求原文的 UTF-8 字节坐标。
/// 字形顺序沿用服务输出，不承诺按源偏移排序；一个簇可以对应多个字形。
/// 参见 [`TextShapeResult`](super::TextShapeResult) 与 [`TextShapeRequest::text`](super::TextShapeRequest::text)。
#[derive(Clone, Debug, PartialEq)]
pub struct TextShapeRun {
    /// 该硬行覆盖的请求原文半开字节范围；起点相对请求文本，允许包含保留的硬分隔符。
    pub source_range: Range<usize>,
    /// 本条目的代表方向。Runtime 投影取首字形方向，空行回退整体解析方向；不保证逐字形一致。
    pub direction: TextDirection,
    /// 本行自有的中立字形集合；不携带请求文本的借用。
    pub glyphs: Vec<TextGlyph>,
}
