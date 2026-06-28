use super::kinsoku::apply_kinsoku_start_rules;
use crate::graphics::text::shaping::shape_horizontal_line;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

const SOFT_HYPHEN: char = '\u{00ad}';
const SOFT_HYPHEN_BREAK_SUFFIX: &str = "-";
const NON_BREAKING_SPACE: char = '\u{00a0}';

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineBreakChunk<'a> {
    pub text: &'a str,
    pub visual_range: UiTextRange,
    pub source_range: UiTextRange,
    pub allow_glyph_fallback: bool,
    pub break_suffix: Option<LineBreakSuffix>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineBreakSuffix {
    pub text: &'static str,
    pub source_range: UiTextRange,
}

pub(crate) fn line_break_chunks<'a>(
    text: &'a str,
    style: &UiResolvedStyle,
) -> Vec<LineBreakChunk<'a>> {
    if text.is_empty() {
        return Vec::new();
    }

    let shaped = shape_horizontal_line(
        text,
        style,
        UiTextDirection::Auto,
        UiTextRange {
            start: 0,
            end: text.len(),
        },
    );
    let mut chunks = Vec::new();
    let mut chunk_start = 0;

    for line in &shaped.lines {
        for glyph in &line.glyphs {
            if !glyph.cluster_flags.cluster_start || !glyph.cluster_flags.soft_break {
                continue;
            }

            let chunk_end = glyph.visual_range.end.min(text.len());
            if chunk_end <= chunk_start || !text.is_char_boundary(chunk_end) {
                continue;
            }

            push_chunk(text, chunk_start, chunk_end, &mut chunks);
            chunk_start = chunk_end;
        }
    }

    if chunk_start < text.len() {
        push_chunk(text, chunk_start, text.len(), &mut chunks);
    }

    apply_kinsoku_start_rules(text, chunks)
}

fn push_chunk<'a>(
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
        let mut chunk = LineBreakChunk::new(
            &text[chunk_start..chunk_end],
            UiTextRange {
                start: chunk_start,
                end: chunk_end,
            },
            UiTextRange {
                start: chunk_start,
                end: chunk_end,
            },
            None,
        );
        chunk.allow_glyph_fallback = !chunk_text.contains(NON_BREAKING_SPACE);
        chunks.push(chunk);
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
                UiTextRange {
                    start: visible_start,
                    end: soft_hyphen_start,
                },
                UiTextRange {
                    start: visible_start,
                    end: soft_hyphen_start,
                },
                Some(LineBreakSuffix {
                    text: SOFT_HYPHEN_BREAK_SUFFIX,
                    source_range: UiTextRange {
                        start: soft_hyphen_start,
                        end: soft_hyphen_end,
                    },
                }),
            ));
        }
        visible_start = soft_hyphen_end;
    }

    if visible_start < chunk_end {
        chunks.push(LineBreakChunk::new(
            &text[visible_start..chunk_end],
            UiTextRange {
                start: visible_start,
                end: chunk_end,
            },
            UiTextRange {
                start: visible_start,
                end: chunk_end,
            },
            None,
        ));
    }
}

impl<'a> LineBreakChunk<'a> {
    fn new(
        text: &'a str,
        visual_range: UiTextRange,
        source_range: UiTextRange,
        break_suffix: Option<LineBreakSuffix>,
    ) -> Self {
        Self {
            text,
            visual_range,
            source_range,
            allow_glyph_fallback: true,
            break_suffix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::line_break_chunks;
    use zircon_runtime_interface::ui::surface::UiResolvedStyle;

    #[test]
    fn line_break_chunks_keep_cjk_open_punctuation_with_following_text() {
        let chunks = line_break_chunks("中（文", &UiResolvedStyle::default());
        let texts: Vec<_> = chunks.iter().map(|chunk| chunk.text).collect();

        assert_eq!(texts, vec!["中", "（文"]);
        assert!(!chunks[1].allow_glyph_fallback);
    }
}
