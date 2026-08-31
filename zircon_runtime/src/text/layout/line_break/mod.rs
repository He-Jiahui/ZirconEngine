pub(crate) use self::boundary_correction::{
    BOUNDARY_SHAPING_CONTEXT_GRAPHEMES, corrected_glyph_ranges_with_provider,
    corrected_index_advance_with_provider, corrected_metric_ranges,
};
use self::glue::allows_glyph_fallback;
pub(crate) use self::greedy::{line_text_fits_with_provider, should_wrap_before_accumulated};
pub(crate) use self::soft_hyphen::{
    DiscretionaryHyphenDecision, break_suffix_at as soft_hyphen_break_suffix_at,
};
pub(crate) use self::wrap_space::{trailing_wrap_space_byte_len, trim_leading_wrap_spaces};
use super::kinsoku::apply_kinsoku_start_rules;
use crate::core::framework::text::{TextDirection, TextLayoutError};
#[cfg(test)]
use crate::text::shaping::DirectTextShapeRunProvider;
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome};
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
    pub break_suffix: Option<DiscretionaryHyphenDecision>,
}

#[cfg(test)]
pub(crate) fn line_break_chunks<'a>(text: &'a str, style: &TextStyle) -> Vec<LineBreakChunk<'a>> {
    let mut provider = DirectTextShapeRunProvider::default();
    line_break_chunks_with_provider(text, style, &mut provider)
        .into_result()
        .expect("test shaping request must be valid")
}

pub(crate) fn line_break_chunks_with_provider<'a, P>(
    text: &'a str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<LineBreakChunk<'a>>>
where
    P: TextShapeRunProvider + ?Sized,
{
    if text.is_empty() {
        return TextShapingOutcome::Ready(Vec::new());
    }

    let mut chunks = Vec::new();
    for hard_line in crate::text::hard_lines(text) {
        let source_range = hard_line.source_range();
        let Some(paragraph_text) = text.get(source_range.clone()) else {
            return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
        };
        let shaped = match provider.shape_horizontal_range_with_kerning(
            paragraph_text,
            style,
            TextDirection::Auto,
            TextRange {
                start: source_range.start,
                end: source_range.end,
            },
            true,
        ) {
            TextShapingOutcome::Ready(shaped) => shaped,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        match append_shaped_line_break_chunks(text, &shaped, source_range, &mut chunks) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
    }

    TextShapingOutcome::Ready(apply_kinsoku_start_rules(text, chunks))
}

fn append_shaped_line_break_chunks<'a>(
    text: &'a str,
    shaped: &crate::text::ShapedGlyphRun,
    source_range: std::ops::Range<usize>,
    chunks: &mut Vec<LineBreakChunk<'a>>,
) -> TextLayoutOutcome<()> {
    if source_range.start > source_range.end
        || source_range.end > text.len()
        || !text.is_char_boundary(source_range.start)
        || !text.is_char_boundary(source_range.end)
    {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    }

    // Shaping backends may expose glyphs in visual order (notably RTL runs). Collect only
    // logical break boundaries first, then materialize chunks in source order.
    let mut break_boundaries = Vec::new();
    for line in &shaped.lines {
        for glyph in &line.glyphs {
            if !glyph.cluster_flags.cluster_start {
                continue;
            }

            let glyph_range = glyph.source_range;
            if glyph_range.start == glyph_range.end && glyph.cluster_flags.virtual_glyph {
                if glyph_range.start < source_range.start
                    || glyph_range.start > source_range.end
                    || !text.is_char_boundary(glyph_range.start)
                {
                    return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
                }
                continue;
            }
            if glyph_range.start < source_range.start
                || glyph_range.end > source_range.end
                || glyph_range.start >= glyph_range.end
                || !text.is_char_boundary(glyph_range.start)
                || !text.is_char_boundary(glyph_range.end)
            {
                return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
            }

            if !glyph.cluster_flags.soft_break && !glyph.cluster_flags.mandatory_break {
                continue;
            }

            break_boundaries.push((
                glyph_range.start,
                glyph_range.end,
                glyph.cluster_flags.mandatory_break,
            ));
        }
    }

    if break_boundaries.windows(2).any(|boundaries| {
        boundaries[0].1 > boundaries[1].1
            || (boundaries[0].1 == boundaries[1].1 && boundaries[0].0 > boundaries[1].0)
    }) {
        break_boundaries.sort_unstable_by_key(|boundary| (boundary.1, boundary.0));
    }
    let mut chunk_start = source_range.start;
    let mut previous_cluster_end = source_range.start;
    let mut boundary_index = 0;
    while boundary_index < break_boundaries.len() {
        let (cluster_start, chunk_end, mut mandatory_break) = break_boundaries[boundary_index];
        if cluster_start < previous_cluster_end {
            return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
        }
        boundary_index += 1;
        while let Some(&(next_start, next_end, next_mandatory)) =
            break_boundaries.get(boundary_index)
        {
            if next_end != chunk_end {
                break;
            }
            if next_start != cluster_start {
                return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
            }
            mandatory_break |= next_mandatory;
            boundary_index += 1;
        }
        if chunk_end <= chunk_start {
            return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
        }
        soft_hyphen::push_chunks(text, chunk_start, chunk_end, chunks);
        if mandatory_break {
            if let Some(chunk) = chunks.last_mut() {
                chunk.mandatory_break = true;
            }
        }
        chunk_start = chunk_end;
        previous_cluster_end = chunk_end;
    }

    if chunk_start < source_range.end {
        soft_hyphen::push_chunks(text, chunk_start, source_range.end, chunks);
    }
    TextShapingOutcome::Ready(())
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
) -> TextLayoutOutcome<Vec<LineBreakChunk<'a>>>
where
    P: TextShapeRunProvider + ?Sized,
{
    line_break_chunks_with_provider(text, style, provider)
        .map(|chunks| smart::apply_word_smart_rules(text, chunks))
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
        break_suffix: Option<DiscretionaryHyphenDecision>,
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
