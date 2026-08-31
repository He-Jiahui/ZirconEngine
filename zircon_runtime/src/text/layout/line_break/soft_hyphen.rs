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
mod tests {
    use super::{DiscretionaryHyphenMarker, SOFT_HYPHEN, break_suffix_at, push_chunks};
    use crate::text::TextRange;
    use crate::text::layout::line_break::LineBreakChunk;

    #[test]
    fn plain_chunk_stays_single_chunk_without_break_suffix() {
        let text = "prefix";
        let mut chunks = Vec::new();

        push_chunks(text, 0, text.len(), &mut chunks);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, text);
        assert!(chunks[0].break_suffix.is_none());
    }

    #[test]
    fn soft_hyphen_is_removed_from_visual_text_and_exposed_as_break_suffix() {
        let text = "pre\u{00ad}fix";
        let mut chunks = Vec::<LineBreakChunk<'_>>::new();

        push_chunks(text, 0, text.len(), &mut chunks);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].text, "pre");
        assert_eq!(chunks[0].source_range, TextRange { start: 0, end: 3 });
        let suffix = chunks[0].break_suffix.expect("soft-hyphen suffix");
        assert_eq!(suffix.marker(), DiscretionaryHyphenMarker::HyphenMinus);
        assert_eq!(suffix.marker_text(), "-");
        assert_eq!(
            suffix.consumed_source_range(),
            TextRange {
                start: 3,
                end: 3 + SOFT_HYPHEN.len_utf8()
            }
        );
        assert_eq!(suffix.virtual_anchor(), 3 + SOFT_HYPHEN.len_utf8());
        assert_eq!(chunks[1].text, "fix");
        assert_eq!(
            chunks[1].source_range,
            TextRange {
                start: 3 + SOFT_HYPHEN.len_utf8(),
                end: text.len()
            }
        );
        assert!(chunks[1].break_suffix.is_none());
    }

    #[test]
    fn each_visible_soft_hyphen_prefix_receives_its_own_break_suffix() {
        let text = "a\u{00ad}b\u{00ad}c";
        let mut chunks = Vec::<LineBreakChunk<'_>>::new();

        push_chunks(text, 0, text.len(), &mut chunks);

        let texts: Vec<_> = chunks.iter().map(|chunk| chunk.text).collect();
        assert_eq!(texts, vec!["a", "b", "c"]);
        assert_eq!(
            chunks[0].break_suffix.expect("first suffix").marker_text(),
            "-"
        );
        assert_eq!(
            chunks[1].break_suffix.expect("second suffix").marker_text(),
            "-"
        );
        assert!(chunks[2].break_suffix.is_none());
    }

    #[test]
    fn suffix_lookup_recovers_visual_hyphen_and_hidden_source_range() {
        let text = "pre\u{00ad}fix";

        let suffix = break_suffix_at(text, 3).expect("soft-hyphen suffix");

        assert_eq!(suffix.marker_text(), "-");
        assert_eq!(
            suffix.consumed_source_range(),
            TextRange { start: 3, end: 5 }
        );
        assert_eq!(suffix.virtual_anchor(), 5);
        let rebased = suffix
            .rebased(7)
            .expect("source offset remains representable");
        assert_eq!(
            rebased.consumed_source_range(),
            TextRange { start: 10, end: 12 }
        );
        assert_eq!(rebased.virtual_anchor(), 12);
        assert!(break_suffix_at(text, 0).is_none());
    }
}
