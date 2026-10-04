use super::LineBreakChunk;
use crate::text::TextRange;

const SOFT_HYPHEN: char = '\u{00ad}';
const SOFT_HYPHEN_BREAK_SUFFIX: &str = "-";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiscretionaryHyphenMarker {
    HyphenMinus,
}

impl DiscretionaryHyphenMarker {
    pub(crate) const fn text(self) -> &'static str {
        match self {
            Self::HyphenMinus => SOFT_HYPHEN_BREAK_SUFFIX,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DiscretionaryHyphenDecision {
    marker: DiscretionaryHyphenMarker,
    consumed_source_range: TextRange,
    virtual_anchor: usize,
}

impl DiscretionaryHyphenDecision {
    fn from_soft_hyphen(source_range: TextRange) -> Self {
        Self {
            marker: DiscretionaryHyphenMarker::HyphenMinus,
            virtual_anchor: source_range.end,
            consumed_source_range: source_range,
        }
    }

    #[cfg(test)]
    pub(crate) const fn marker(self) -> DiscretionaryHyphenMarker {
        self.marker
    }

    pub(crate) const fn marker_text(self) -> &'static str {
        self.marker.text()
    }

    pub(crate) const fn consumed_source_range(self) -> TextRange {
        self.consumed_source_range
    }

    pub(crate) const fn virtual_anchor(self) -> usize {
        self.virtual_anchor
    }

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

pub(crate) fn break_suffix_at(text: &str, break_end: usize) -> Option<DiscretionaryHyphenDecision> {
    let source_end = break_end.saturating_add(SOFT_HYPHEN.len_utf8());
    (text.get(break_end..source_end) == Some("\u{00ad}")).then(|| {
        DiscretionaryHyphenDecision::from_soft_hyphen(TextRange {
            start: break_end,
            end: source_end,
        })
    })
}

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
