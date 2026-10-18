//! 对已生成的换行候选块应用词尾标点保护。
//!
//! 父入口仅在基础分块 Ready 后调用；本模块不请求成形或选择最终行宽。
//! 输入文本保持借用，输出拥有新的块向量；范围仍是同一次输入的 UTF-8 字节坐标。
//! 触发类别限于 OtherPunctuation，保护尾串另接纳 ClosePunctuation 与 FinalPunctuation，
//! 不代表完整禁则或所有标点类别。

use super::LineBreakChunk;
use crate::text::{TextRange, WordBoundaryMap};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

/// 在整形给出的断行块上补充词尾标点策略，避免标点独占下一视觉行。
/// 只合并连续且源到显示字节一一对应的块；硬行边界与生成字符不得在此重映射。
///
/// # 参数、返回与生命周期
/// `text` 是本次输入，`chunks` 是按源序生成、与该输入对应的块向量，由本函数消费。
/// 返回调整后的向量；块内字符串仍借用 `text`，不会持有 provider 或成形结果。
///
/// # 约束与参考
/// 调用方保证既有端点合法、文本借用及坐标基准匹配；等长筛选并不完整验证映射身份。
/// 同一词尾游标由有序处理链共享，查询允许重复偏移，但不可逆序。参见 [WordEndCursor]。
pub(super) fn apply_word_smart_rules<'a>(
    text: &'a str,
    chunks: Vec<LineBreakChunk<'a>>,
) -> Vec<LineBreakChunk<'a>> {
    let mut adjusted: Vec<LineBreakChunk<'a>> = Vec::with_capacity(chunks.len());
    let mut word_ends = WordEndCursor::new(WordBoundaryMap::new(text).ranges());

    for chunk in chunks {
        append_word_smart_chunk(text, chunk, &mut adjusted, &mut word_ends);
    }

    adjusted
}

/// 顺序处理一个块，先拆前导保护串，再找内部词尾，最后提交叶块。
/// `adjusted` 保存已处理的前缀，`word_ends` 共享仅向前消费的词尾状态；拆分后继续处理余块。
/// 调用方提供与 `text` 对应的合法块；处理顺序维持游标查询非递减，不提供独立回退协议。
fn append_word_smart_chunk<'a, I>(
    text: &'a str,
    mut chunk: LineBreakChunk<'a>,
    adjusted: &mut Vec<LineBreakChunk<'a>>,
    word_ends: &mut WordEndCursor<I>,
) where
    I: Iterator<Item = TextRange>,
{
    loop {
        if let Some((prefix, suffix)) = split_after_leading_trailing_punctuation(text, chunk) {
            append_word_smart_leaf(text, prefix, adjusted, word_ends);
            chunk = suffix;
            continue;
        }

        if let Some((prefix, suffix)) =
            split_after_internal_trailing_punctuation(text, chunk, word_ends)
        {
            append_word_smart_leaf(text, prefix, adjusted, word_ends);
            chunk = suffix;
            continue;
        }

        break;
    }

    append_word_smart_leaf(text, chunk, adjusted, word_ends);
}

/// 若块以触发标点开头且保护尾串之后仍有文本，则在尾串末端尝试拆分。
/// 返回前后两个借用原输入的块；无首字符、无触发类别、无剩余文本或拆分前提不满足时返回 None。
fn split_after_leading_trailing_punctuation<'a>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)> {
    let (offset, ch) = chunk.text.char_indices().next()?;
    if offset != 0 || !is_word_smart_trailing_punctuation(ch) {
        return None;
    }

    let split_offset = word_smart_protected_run_end(chunk.text, offset, ch);
    if split_offset >= chunk.text.len() {
        return None;
    }

    split_word_smart_chunk(text, chunk, split_offset)
}

/// 将不可再拆的块并入已处理前缀，或追加为新叶块。
/// 合并要求前块不是强制断点且两套范围均连续；重新切片仍借用同一 `text`。
/// 已有后缀优先，强制断点取并集；合并或词尾保护成功会关闭该块的字素折行许可。
fn append_word_smart_leaf<'a, I>(
    text: &'a str,
    mut chunk: LineBreakChunk<'a>,
    adjusted: &mut Vec<LineBreakChunk<'a>>,
    word_ends: &mut WordEndCursor<I>,
) where
    I: Iterator<Item = TextRange>,
{
    if starts_with_word_smart_trailing_punctuation(chunk.text)
        && word_ends.has_word_ending_at(chunk.visual_range.start)
    {
        if let Some(previous) = adjusted.last_mut().filter(|previous| {
            !previous.mandatory_break
                && has_isomorphic_source_mapping(previous)
                && has_isomorphic_source_mapping(&chunk)
                && previous.visual_range.end == chunk.visual_range.start
                && previous.source_range.end == chunk.source_range.start
        }) {
            let start = previous.visual_range.start;
            let end = chunk.visual_range.end;
            if start < end
                && end <= text.len()
                && text.is_char_boundary(start)
                && text.is_char_boundary(end)
            {
                previous.text = &text[start..end];
                previous.visual_range.end = end;
                previous.source_range.end = chunk.source_range.end;
                // 前块已有的断点后缀优先；仅在缺失时接收后块携带的后缀。
                if previous.break_suffix.is_none() {
                    previous.break_suffix = chunk.break_suffix;
                }
                previous.mandatory_break |= chunk.mandatory_break;
                previous.allow_glyph_fallback = false;
                return;
            }
        }
    }

    if trailing_punctuation_suffix_start(chunk.text).is_some_and(|offset| {
        word_ends.has_word_ending_at(chunk.visual_range.start.saturating_add(offset))
    }) {
        chunk.allow_glyph_fallback = false;
    }

    adjusted.push(chunk);
}

/// 顺序扫描内部触发标点，在完整词尾之后、保护串之后仍有余文时尝试拆分。
/// `word_ends` 与前后叶块共享状态；首个适用位置决定本次拆分，余块交外层继续处理。
fn split_after_internal_trailing_punctuation<'a, I>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
    word_ends: &mut WordEndCursor<I>,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)>
where
    I: Iterator<Item = TextRange>,
{
    for (offset, ch) in chunk.text.char_indices() {
        let split_offset = word_smart_protected_run_end(chunk.text, offset, ch);
        if offset == 0
            || split_offset >= chunk.text.len()
            || !is_word_smart_trailing_punctuation(ch)
        {
            continue;
        }
        let source_offset = chunk.visual_range.start.saturating_add(offset);
        if !word_ends.has_word_ending_at(source_offset) {
            continue;
        }

        return split_word_smart_chunk(text, chunk, split_offset);
    }

    None
}

/// 按相对字节偏移构造前后两个块，保留对原输入的借用。
///
/// `split_offset` 相对于当前块；局部检查包括等长筛选、拆分位置及拆分点 UTF-8 边界。
/// 原端点、文本身份与坐标基准仍由调用方保证，显示与源的绝对起点不必相等。
/// 返回 None 表示局部拆分前提不满足；前块不携带强制断点或后缀，后块继承二者。
/// 两块继承原字素折行许可，之后的叶块处理可以进一步关闭许可。
fn split_word_smart_chunk<'a>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
    split_offset: usize,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)> {
    let visual_len = chunk
        .visual_range
        .end
        .saturating_sub(chunk.visual_range.start);
    if !has_isomorphic_source_mapping(&chunk) || split_offset >= visual_len {
        return None;
    }

    let split = chunk.visual_range.start + split_offset;
    let source_split = chunk.source_range.start + split_offset;
    if split >= chunk.visual_range.end || source_split > chunk.source_range.end {
        return None;
    }
    if !text.is_char_boundary(split) {
        return None;
    }

    let prefix = LineBreakChunk {
        text: &text[chunk.visual_range.start..split],
        visual_range: TextRange {
            start: chunk.visual_range.start,
            end: split,
        },
        source_range: TextRange {
            start: chunk.source_range.start,
            end: source_split,
        },
        allow_glyph_fallback: chunk.allow_glyph_fallback,
        mandatory_break: false,
        break_suffix: None,
    };
    let suffix = LineBreakChunk {
        text: &text[split..chunk.visual_range.end],
        visual_range: TextRange {
            start: split,
            end: chunk.visual_range.end,
        },
        source_range: TextRange {
            start: source_split,
            end: chunk.source_range.end,
        },
        allow_glyph_fallback: chunk.allow_glyph_fallback,
        mandatory_break: chunk.mandatory_break,
        break_suffix: chunk.break_suffix,
    };

    Some((prefix, suffix))
}

/// 仅比较两套范围的饱和长度与块文本字节长度，不检查端点、文本内容或坐标身份。
/// 此谓词只是有序流水线的等长筛选；不能替代调用方对合法输入映射的保证。
fn has_isomorphic_source_mapping(chunk: &LineBreakChunk<'_>) -> bool {
    let visual_len = chunk
        .visual_range
        .end
        .saturating_sub(chunk.visual_range.start);
    let source_len = chunk
        .source_range
        .end
        .saturating_sub(chunk.source_range.start);
    visual_len == source_len && visual_len == chunk.text.len()
}

fn starts_with_word_smart_trailing_punctuation(text: &str) -> bool {
    text.chars()
        .next()
        .is_some_and(is_word_smart_trailing_punctuation)
}

/// 从块尾反向查找保护尾串中的首个触发标点，返回其相对 UTF-8 字节偏移。
/// 关闭/结束标点可延长尾串，但自身不触发保护；偏移为零的触发标点由前导路径处理。
fn trailing_punctuation_suffix_start(text: &str) -> Option<usize> {
    let mut trigger_start = None;
    for (offset, ch) in text.char_indices().rev() {
        if is_word_smart_trailing_extension(ch) {
            if is_word_smart_trailing_punctuation(ch) && offset > 0 {
                trigger_start = Some(offset);
            }
            continue;
        }
        break;
    }

    trigger_start
}

/// 从已知字符位置向后收集连续保护尾串，返回相对当前块的末端字节偏移。
/// `offset` 与 `ch` 必须来自该文本的同一次字符迭代；不对任意外部偏移重新校验。
fn word_smart_protected_run_end(text: &str, offset: usize, ch: char) -> usize {
    let mut end = offset + ch.len_utf8();
    if !is_word_smart_trailing_punctuation(ch) {
        return end;
    }

    while let Some(next) = text[end..].chars().next() {
        if !is_word_smart_trailing_extension(next) {
            break;
        }
        end += next.len_utf8();
    }

    end
}

fn is_word_smart_trailing_punctuation(ch: char) -> bool {
    matches!(ch.general_category(), GeneralCategory::OtherPunctuation)
}

fn is_word_smart_trailing_extension(ch: char) -> bool {
    matches!(
        ch.general_category(),
        GeneralCategory::OtherPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::FinalPunctuation
    )
}

/// 对有序词范围进行非递减字节偏移查询的共享游标。
///
/// 创建时预取一个范围，后续只丢弃结束位置早于查询的范围；相等偏移可重复查询。
/// 该状态只在本次调整调用内存活，不拥有文本，也不提供重置、回退或错误返回。
struct WordEndCursor<I> {
    /// 尚未消费的词范围迭代器，只向前推进。
    ranges: I,
    /// 已预取的下一个范围，保留给当前位置及后续查询。
    next: Option<TextRange>,
    /// 最近的查询偏移，供调试断言诊断逆序调用。
    last_query: usize,
}

impl<I> WordEndCursor<I>
where
    I: Iterator<Item = TextRange>,
{
    /// 接收按结束位置有序的范围迭代器并预取首项；空迭代器保持无候选状态。
    /// 返回游标拥有迭代器；若迭代器借用文本，其有效期仍由外层输入借用约束。
    fn new(mut ranges: I) -> Self {
        let next = ranges.next();
        Self {
            ranges,
            next,
            last_query: 0,
        }
    }

    /// 判断是否存在恰好结束于 `offset` 的词，不判断该偏移是否位于某个词内。
    ///
    /// `offset` 使用同一次输入的 UTF-8 字节坐标，必须不小于此前查询，可与其相等。
    /// 调试断言仅诊断逆序；关闭调试断言时仍无回退、恢复或错误返回协议。
    /// 本次调用可能消费更早的范围，返回布尔值不转移范围或文本所有权。
    fn has_word_ending_at(&mut self, offset: usize) -> bool {
        debug_assert!(offset >= self.last_query);
        self.last_query = offset;
        while self.next.is_some_and(|range| range.end < offset) {
            self.next = self.ranges.next();
        }
        self.next.is_some_and(|range| range.end == offset)
    }
}

#[cfg(test)]
#[path = "smart/tests/cases.rs"]
mod tests;
