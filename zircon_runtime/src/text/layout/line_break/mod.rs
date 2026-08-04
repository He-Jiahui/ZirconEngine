pub(crate) use self::boundary_correction::{
    corrected_glyph_ranges_with_provider, corrected_index_advance_with_provider,
    corrected_metric_ranges, BOUNDARY_SHAPING_CONTEXT_GRAPHEMES,
};
use self::glue::allows_glyph_fallback;
pub(crate) use self::greedy::{line_text_fits_with_provider, should_wrap_before_accumulated};
pub(crate) use self::soft_hyphen::{
    break_suffix_at as soft_hyphen_break_suffix_at, LineBreakSuffix,
};
pub(crate) use self::wrap_space::{trailing_wrap_space_byte_len, trim_leading_wrap_spaces};
use super::kinsoku::apply_kinsoku_start_rules;
use crate::core::framework::text::TextDirection;
#[cfg(test)]
use crate::text::shaping::DirectTextShapeRunProvider;
use crate::text::shaping::TextShapeRunProvider;
use crate::text::{TextRange, TextStyle};

mod boundary_correction;
mod glue;
mod glyph_fallback;
mod greedy;
mod smart;
mod soft_hyphen;
mod wrap_space;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineBreakChunk<'a> {
    pub text: &'a str,
    pub visual_range: TextRange,
    pub source_range: TextRange,
    pub allow_glyph_fallback: bool,
    pub mandatory_break: bool,
    pub break_suffix: Option<LineBreakSuffix>,
}

#[cfg(test)]
pub(crate) fn line_break_chunks<'a>(text: &'a str, style: &TextStyle) -> Vec<LineBreakChunk<'a>> {
    let mut provider = DirectTextShapeRunProvider;
    line_break_chunks_with_provider(text, style, &mut provider)
}

pub(crate) fn line_break_chunks_with_provider<'a, P>(
    text: &'a str,
    style: &TextStyle,
    provider: &mut P,
) -> Vec<LineBreakChunk<'a>>
where
    P: TextShapeRunProvider + ?Sized,
{
    if text.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    for hard_line in crate::text::hard_lines(text) {
        let source_range = hard_line.source_range();
        let Some(paragraph_text) = text.get(source_range.clone()) else {
            continue;
        };
        let shaped = provider.shape_horizontal_line_with_kerning(
            paragraph_text,
            style,
            TextDirection::Auto,
            TextRange {
                start: source_range.start,
                end: source_range.end,
            },
            true,
        );
        let paragraph_chunk_start = chunks.len();
        append_shaped_line_break_chunks(text, &shaped, source_range, &mut chunks);
        if hard_line.is_run_cap_break() {
            if chunks.len() == paragraph_chunk_start {
                let source_range = hard_line.content;
                chunks.push(LineBreakChunk::new(
                    &text[source_range.clone()],
                    TextRange {
                        start: source_range.start,
                        end: source_range.end,
                    },
                    TextRange {
                        start: source_range.start,
                        end: source_range.end,
                    },
                    None,
                ));
            }
            if let Some(chunk) = chunks.last_mut() {
                chunk.mandatory_break = true;
            }
        }
    }

    apply_kinsoku_start_rules(text, chunks)
}

fn append_shaped_line_break_chunks<'a>(
    text: &'a str,
    shaped: &crate::text::ShapedGlyphRun,
    source_range: std::ops::Range<usize>,
    chunks: &mut Vec<LineBreakChunk<'a>>,
) {
    let mut chunk_start = source_range.start;
    for line in &shaped.lines {
        for glyph in &line.glyphs {
            if !glyph.cluster_flags.cluster_start
                || (!glyph.cluster_flags.soft_break && !glyph.cluster_flags.mandatory_break)
            {
                continue;
            }

            let chunk_end = glyph.source_range.end.min(source_range.end);
            if chunk_end <= chunk_start || !text.is_char_boundary(chunk_end) {
                continue;
            }

            soft_hyphen::push_chunks(text, chunk_start, chunk_end, chunks);
            if glyph.cluster_flags.mandatory_break {
                if let Some(chunk) = chunks.last_mut() {
                    chunk.mandatory_break = true;
                }
            }
            chunk_start = chunk_end;
        }
    }

    if chunk_start < source_range.end {
        soft_hyphen::push_chunks(text, chunk_start, source_range.end, chunks);
    }
}

#[cfg(test)]
pub(crate) fn word_smart_line_break_chunks<'a>(
    text: &'a str,
    style: &TextStyle,
) -> Vec<LineBreakChunk<'a>> {
    smart::apply_word_smart_rules(text, line_break_chunks(text, style))
}

pub(crate) fn word_smart_line_break_chunks_with_provider<'a, P>(
    text: &'a str,
    style: &TextStyle,
    provider: &mut P,
) -> Vec<LineBreakChunk<'a>>
where
    P: TextShapeRunProvider + ?Sized,
{
    smart::apply_word_smart_rules(text, line_break_chunks_with_provider(text, style, provider))
}

impl<'a> LineBreakChunk<'a> {
    pub(crate) fn should_fallback_to_glyph_wrap_with_advance(
        &self,
        candidate_text: &str,
        candidate_advance: f32,
        max_width: f32,
    ) -> bool {
        glyph_fallback::should_fallback_to_glyph_wrap_with_advance(
            self.allow_glyph_fallback,
            candidate_text,
            candidate_advance,
            max_width,
        )
    }

    #[cfg(test)]
    pub(crate) fn should_fallback_to_glyph_wrap(
        &self,
        candidate_text: &str,
        max_width: f32,
        style: &TextStyle,
    ) -> bool {
        glyph_fallback::should_fallback_to_glyph_wrap(
            self.allow_glyph_fallback,
            candidate_text,
            max_width,
            style,
        )
    }

    fn new(
        text: &'a str,
        visual_range: TextRange,
        source_range: TextRange,
        break_suffix: Option<LineBreakSuffix>,
    ) -> Self {
        Self {
            text,
            visual_range,
            source_range,
            allow_glyph_fallback: allows_glyph_fallback(text),
            mandatory_break: false,
            break_suffix,
        }
    }
}

#[cfg(test)]
mod tests;
