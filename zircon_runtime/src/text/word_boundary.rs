//! 原文借用上的词边界查询，供导航和完整词前缀裁切共用。
//! 偏移与返回范围均使用本次输入的 UTF-8 字节坐标；不负责按显示宽度适配文本。

use unicode_segmentation::UnicodeSegmentation;

use super::{compiled_unicode_data_snapshot_id, TextRange, UnicodeDataSnapshotId};

/// Zero-copy UAX #29 word-boundary view tied to the Unicode data identity that produced it.
///
/// The view deliberately retains the source string instead of materializing every boundary. UI
/// navigation and one-shot overflow queries therefore share one policy owner without adding a
/// second allocation proportional to paragraph length. A retained paragraph analysis can cache a
/// materialized form later without changing the query semantics defined here.
/// 复制该视图只复制借用和快照标识，原文仍须在整个查询或迭代期间有效。
/// 标识记录调用方声明的数据身份；实际分词仍由下方的 Unicode 分词 trait 完成。
#[derive(Clone, Copy, Debug)]
pub(crate) struct WordBoundaryMap<'text> {
    /// 所有返回范围对应的原文；视图不持有字符串所有权。
    text: &'text str,
    /// 随视图保存的 Unicode 数据身份，不是用于切换分词实现的配置。
    unicode_data_snapshot: UnicodeDataSnapshotId,
}

impl<'text> WordBoundaryMap<'text> {
    /// 借用原文，并记录当前编译的 Unicode 数据身份。
    ///
    /// text 须覆盖视图及其派生迭代器的使用期；返回值不复制原文。
    /// 显式指定身份时使用 [Self::for_snapshot]。
    pub(crate) fn new(text: &'text str) -> Self {
        Self::for_snapshot(text, compiled_unicode_data_snapshot_id())
    }

    /// 将原文借用与给定身份组装为视图。
    ///
    /// unicode_data_snapshot 仅被保存，不在此校验，也不选择对应版本的分词数据。
    /// 返回值与 text 共享借用期限；需要身份一致性时由调用方保证。
    pub(crate) const fn for_snapshot(
        text: &'text str,
        unicode_data_snapshot: UnicodeDataSnapshotId,
    ) -> Self {
        Self {
            text,
            unicode_data_snapshot,
        }
    }

    /// 返回创建视图时保存的身份值；该查询不重新读取或验证分词数据。
    pub(crate) const fn unicode_data_snapshot(self) -> UnicodeDataSnapshotId {
        self.unicode_data_snapshot
    }

    /// 查找起点严格早于 offset 的最后一个词，返回其原文字节起点。
    ///
    /// offset 先限制到原文长度，再向下对齐 UTF-8 字符边界；这不是字素对齐。
    /// 没有符合条件的词时返回 None，各次查询不要求偏移单调。
    pub(crate) fn previous_word_start(self, offset: usize) -> Option<usize> {
        let offset = floor_utf8_boundary(self.text, offset);
        self.ranges()
            .rev()
            .find(|range| range.start < offset)
            .map(|range| range.start)
    }

    /// 查找终点严格晚于 offset 的第一个词，返回其原文字节终点。
    ///
    /// 输入偏移与 [Self::previous_word_start] 使用相同的 UTF-8 向下对齐规则。
    /// 没有符合条件的词时返回 None。
    pub(crate) fn next_word_end(self, offset: usize) -> Option<usize> {
        let offset = floor_utf8_boundary(self.text, offset);
        self.ranges()
            .find(|range| range.end > offset)
            .map(|range| range.end)
    }

    /// 从前向后查找与归一化偏移相接或覆盖该偏移的词范围。
    ///
    /// offset 使用相同的字节向下对齐规则，且允许等于词尾；相邻词共享端点时
    /// 优先返回前一个词。因此这里不是通常的半开区间包含判断；无匹配时返回 None。
    pub(crate) fn word_range_at(self, offset: usize) -> Option<TextRange> {
        let offset = floor_utf8_boundary(self.text, offset);
        self.ranges()
            .take_while(|range| range.start <= offset)
            .find(|range| offset <= range.end)
    }

    /// Returns the end of the last complete Unicode word within a fitted source prefix.
    ///
    /// Separator and punctuation segments are not retained past the completed word. This keeps an
    /// EndWord marker from publishing dangling whitespace or punctuation as if it were a word.
    /// fitted_end 是调用方已拟合前缀的字节终点，仍会限制长度并向下对齐字符边界。
    /// 返回不晚于它的最后一个完整词终点；没有完整词时返回零，不在这里测量宽度。
    pub(crate) fn completed_prefix_end(self, fitted_end: usize) -> usize {
        let fitted_end = floor_utf8_boundary(self.text, fitted_end);
        self.ranges()
            .take_while(|range| range.end <= fitted_end)
            .last()
            .map(|range| range.end)
            .unwrap_or_default()
    }

    /// 惰性枚举原文中的 Unicode 词范围，分隔符与标点段不作为词返回。
    ///
    /// 返回的双端迭代器借用 text，不借用这个按值传入的视图本体；
    /// 它可在视图值结束后继续使用，但不能超出原文的 'text 生命周期。
    /// 范围是同一原文的 UTF-8 字节位置，不是字素、字形或视觉顺序索引。
    pub(crate) fn ranges(self) -> impl DoubleEndedIterator<Item = TextRange> + 'text {
        self.text
            .unicode_word_indices()
            .map(|(start, word)| TextRange {
                start,
                end: start + word.len(),
            })
    }
}

/// 将字节偏移限制到原文内，并退到不晚于它的 UTF-8 字符边界。
/// 返回值可安全作为该字符串的切片端点；此处不寻找词或字素边界。
fn floor_utf8_boundary(text: &str, offset: usize) -> usize {
    // 导航与完整词前缀查询共用向下取整，保证非边界字节偏移也按同一 UTF-8 规则解释。
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

// 直属夹具只验证下方列出的固定输入，不等同于所有 Unicode 或偏移边界的覆盖。
#[cfg(test)]
#[path = "tests/word_boundary.rs"]
mod tests;
