use self::glue::allows_glyph_fallback;
pub(crate) use self::greedy::{line_text_fits, should_wrap_before_chunk};
pub(crate) use self::soft_hyphen::LineBreakSuffix;
pub(crate) use self::wrap_space::{trailing_wrap_space_byte_len, trim_leading_wrap_spaces};
use super::kinsoku::apply_kinsoku_start_rules;
use crate::graphics::text::shaping::shape_horizontal_line;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

mod glue;
mod glyph_fallback;
mod greedy;
mod smart;
mod soft_hyphen;
mod wrap_space;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineBreakChunk<'a> {
    pub text: &'a str,
    pub visual_range: UiTextRange,
    pub source_range: UiTextRange,
    pub allow_glyph_fallback: bool,
    pub break_suffix: Option<LineBreakSuffix>,
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

            soft_hyphen::push_chunks(text, chunk_start, chunk_end, &mut chunks);
            chunk_start = chunk_end;
        }
    }

    if chunk_start < text.len() {
        soft_hyphen::push_chunks(text, chunk_start, text.len(), &mut chunks);
    }

    apply_kinsoku_start_rules(text, chunks)
}

pub(crate) fn word_smart_line_break_chunks<'a>(
    text: &'a str,
    style: &UiResolvedStyle,
) -> Vec<LineBreakChunk<'a>> {
    smart::apply_word_smart_rules(text, line_break_chunks(text, style))
}

impl<'a> LineBreakChunk<'a> {
    pub(crate) fn should_fallback_to_glyph_wrap(
        &self,
        candidate_text: &str,
        max_width: f32,
        style: &UiResolvedStyle,
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
        visual_range: UiTextRange,
        source_range: UiTextRange,
        break_suffix: Option<LineBreakSuffix>,
    ) -> Self {
        Self {
            text,
            visual_range,
            source_range,
            allow_glyph_fallback: allows_glyph_fallback(text),
            break_suffix,
        }
    }
}

#[cfg(test)]
mod tests;
