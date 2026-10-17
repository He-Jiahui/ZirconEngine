//! 将源中的软连字符拆为隐藏字节范围与可选的可见连字符决定。
//! 分块借用输入文本；是否在断行处显示标记由布局消费方决定。

use super::LineBreakChunk;
use crate::text::TextRange;

// 源 U+00AD 与断行处显示的 ASCII 连字符分别保存，不能用可见标记替换源字节。
const SOFT_HYPHEN: char = '\u{00ad}';
const SOFT_HYPHEN_BREAK_SUFFIX: &str = "-";

/// 断行时补出的可见标记种类；不拥有或改写输入文本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiscretionaryHyphenMarker {
    /// 使用 ASCII 连字符，与源中的 U+00AD 区分。
    HyphenMinus,
}

impl DiscretionaryHyphenMarker {
    /// 返回该标记的静态可见文本；调用方决定测量和插入时机。
    pub(crate) const fn text(self) -> &'static str {
        match self {
            Self::HyphenMinus => SOFT_HYPHEN_BREAK_SUFFIX,
        }
    }
}

/// 隐藏源软连字符的消费范围及可见替代标记决定，不借用或改写文本。
/// 范围和锚点均为 UTF-8 字节偏移；消费方须保持对应文本及坐标基准配对。
/// 参见 [`break_suffix_at`] 与 [`Self::rebased`]；可复制的决定不替代文本边界校验。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DiscretionaryHyphenDecision {
    /// 要显示的标记种类。
    marker: DiscretionaryHyphenMarker,
    /// 被消耗但不直接显示的源字节半开区间，不包含可见前缀。
    consumed_source_range: TextRange,
    /// 供虚拟标记使用的源锚点，等于消耗区间的 `end` 边界。
    virtual_anchor: usize,
}

impl DiscretionaryHyphenDecision {
    /// 保存已确认的软连字符区间；此处不再读取文本校验。
    fn from_soft_hyphen(source_range: TextRange) -> Self {
        Self {
            marker: DiscretionaryHyphenMarker::HyphenMinus,
            virtual_anchor: source_range.end,
            consumed_source_range: source_range,
        }
    }

    #[cfg(test)]
    /// 供直属测试检查标记种类，不改变决定值。
    pub(crate) const fn marker(self) -> DiscretionaryHyphenMarker {
        self.marker
    }

    /// 返回借用静态常量的可见标记文本。
    pub(crate) const fn marker_text(self) -> &'static str {
        self.marker.text()
    }

    /// 返回已消耗的源字节半开区间。
    pub(crate) const fn consumed_source_range(self) -> TextRange {
        self.consumed_source_range
    }

    /// 返回虚拟标记的源锚点；消费方据此构造空源范围。
    pub(crate) const fn virtual_anchor(self) -> usize {
        self.virtual_anchor
    }

    /// 将范围两端及锚点加上 `source_base`，把片段坐标搬到外层坐标基准。
    /// 任一加法溢出返回 `None`；不验证新坐标属于某个 UTF-8 字符串。
    pub(crate) fn rebased(self, source_base: usize) -> Option<Self> {
        Some(Self {
            marker: self.marker,
            consumed_source_range: TextRange {
                start: source_base.checked_add(self.consumed_source_range.start)?,
                end: source_base.checked_add(self.consumed_source_range.end)?,
            },
            virtual_anchor: source_base.checked_add(self.virtual_anchor)?,
        })
    }
}

/// 若 `break_end` 处恰好是 U+00AD，返回消耗该标记的决定，否则返回 `None`。
/// 参数是 `text` 内的字节偏移；越界或非字符边界由安全取片拒绝，不修改文本。
pub(crate) fn break_suffix_at(text: &str, break_end: usize) -> Option<DiscretionaryHyphenDecision> {
    let source_end = break_end.saturating_add(SOFT_HYPHEN.len_utf8());
    (text.get(break_end..source_end) == Some("\u{00ad}")).then(|| {
        DiscretionaryHyphenDecision::from_soft_hyphen(TextRange {
            start: break_end,
            end: source_end,
        })
    })
}

/// 将非空且端点合法的字节区间拆成借用 `text` 的可见块，追加到 `chunks`。
/// 省略 U+00AD 本体，仅给它前方非空可见片段附上后缀决定；不追加空块。
/// 已有块保留，追加可能扩容；非法、空或反向范围直接返回。
pub(super) fn push_chunks<'a>(
    text: &'a str,
    chunk_start: usize,
    chunk_end: usize,
    chunks: &mut Vec<LineBreakChunk<'a>>,
) {
    if chunk_end <= chunk_start
        || !text.is_char_boundary(chunk_start)
        || !text.is_char_boundary(chunk_end)
    {
        return;
    }

    let chunk_text = &text[chunk_start..chunk_end];
    if !chunk_text.contains(SOFT_HYPHEN) {
        chunks.push(LineBreakChunk::new(
            &text[chunk_start..chunk_end],
            TextRange {
                start: chunk_start,
                end: chunk_end,
            },
            TextRange {
                start: chunk_start,
                end: chunk_end,
            },
            None,
        ));
        return;
    }

    // 前导或连续软连字符也推进源游标，但不制造可见空块。
    let mut visible_start = chunk_start;
    for (relative_index, ch) in chunk_text.char_indices() {
        if ch != SOFT_HYPHEN {
            continue;
        }

        let soft_hyphen_start = chunk_start + relative_index;
        let soft_hyphen_end = soft_hyphen_start + SOFT_HYPHEN.len_utf8();
        if visible_start < soft_hyphen_start {
            chunks.push(LineBreakChunk::new(
                &text[visible_start..soft_hyphen_start],
                TextRange {
                    start: visible_start,
                    end: soft_hyphen_start,
                },
                TextRange {
                    start: visible_start,
                    end: soft_hyphen_start,
                },
                Some(DiscretionaryHyphenDecision::from_soft_hyphen(TextRange {
                    start: soft_hyphen_start,
                    end: soft_hyphen_end,
                })),
            ));
        }
        visible_start = soft_hyphen_end;
    }

    // 隐藏标记后若仍有可见尾段，保留原字节范围且不再附加后缀。
    if visible_start < chunk_end {
        chunks.push(LineBreakChunk::new(
            &text[visible_start..chunk_end],
            TextRange {
                start: visible_start,
                end: chunk_end,
            },
            TextRange {
                start: visible_start,
                end: chunk_end,
            },
            None,
        ));
    }
}

#[cfg(test)]
#[path = "tests/soft_hyphen.rs"]
mod tests;
