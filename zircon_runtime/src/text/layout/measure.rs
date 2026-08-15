use std::sync::Arc;

use crate::text::{TextSize, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
use crate::text::TextRange;
use crate::text::{ShapedGlyph, ShapedGlyphRun};

use super::tab::tab_aligned_width;

const DEFAULT_METRICS_SAMPLE: &str = "Hg";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLineMetrics {
    pub width: f32,
    pub baseline: f32,
    pub line_height: f32,
}

pub(crate) fn measure_text_size(text: &str, style: &TextStyle) -> TextSize {
    let mut provider = DirectTextShapeRunProvider;
    measure_text_size_with_provider(text, style, &mut provider)
}

pub(crate) fn measure_text_size_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> TextSize
where
    P: TextShapeRunProvider + ?Sized,
{
    let metrics = line_metrics_with_provider(style, provider);
    let mut width = 0.0_f32;
    let mut line_count = 0_usize;
    crate::text::visit_hard_lines(text, |line| {
        let line = text.get(line.content).unwrap_or_default();
        width = width.max(measure_line_width_with_provider(line, style, provider));
        line_count = line_count.saturating_add(1);
    });
    let line_count = line_count.max(1) as f32;
    TextSize::new(width, metrics.line_height * line_count)
}

pub(crate) fn measure_line_width(text: &str, style: &TextStyle) -> f32 {
    let mut provider = DirectTextShapeRunProvider;
    measure_line_width_with_provider(text, style, &mut provider)
}

pub(crate) fn measure_line_width_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    if text.is_empty() {
        return 0.0;
    }

    if !text.contains('\t') {
        return shape_line_with_provider(text, style, provider).width;
    }

    let grapheme_widths = measured_grapheme_widths_with_provider(text, style, provider);
    tab_aligned_width(
        text,
        &grapheme_widths,
        style,
        shape_line_with_provider(" ", style, provider).width,
    )
}

pub(crate) fn measured_grapheme_widths(text: &str, style: &TextStyle) -> Vec<f32> {
    let mut provider = DirectTextShapeRunProvider;
    measured_grapheme_widths_with_provider(text, style, &mut provider)
}

pub(crate) fn measured_grapheme_widths_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> Vec<f32>
where
    P: TextShapeRunProvider + ?Sized,
{
    let shaped = shape_unconstrained_line_with_provider(text, style, provider);
    measured_grapheme_widths_from_shaped(&shaped, text)
}

pub(crate) fn measure_text_source_range_width(
    text: &str,
    style: &TextStyle,
    range: TextRange,
) -> f32 {
    let mut provider = DirectTextShapeRunProvider;
    measure_text_source_range_width_with_provider(text, style, range, &mut provider)
}

pub(crate) fn measure_text_source_range_width_with_provider<P>(
    text: &str,
    style: &TextStyle,
    range: TextRange,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    measure_text_source_range_width_with_kerning_and_provider(text, style, range, true, provider)
}

#[cfg(test)]
pub(crate) fn measure_text_source_range_width_with_kerning(
    text: &str,
    style: &TextStyle,
    range: TextRange,
    include_kerning: bool,
) -> f32 {
    let mut provider = DirectTextShapeRunProvider;
    measure_text_source_range_width_with_kerning_and_provider(
        text,
        style,
        range,
        include_kerning,
        &mut provider,
    )
}

pub(crate) fn measure_text_source_range_width_with_kerning_and_provider<P>(
    text: &str,
    style: &TextStyle,
    range: TextRange,
    include_kerning: bool,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    if text.is_empty() || range.start >= range.end {
        return 0.0;
    }

    let shaped =
        shape_unconstrained_line_with_kerning_and_provider(text, style, include_kerning, provider);
    measured_width(&shaped, range.start, range.end, include_kerning)
}

pub(crate) fn measured_width(
    run: &ShapedGlyphRun,
    byte_start: usize,
    byte_end: usize,
    include_kerning: bool,
) -> f32 {
    if byte_start >= byte_end {
        return 0.0;
    }
    let source_start = byte_start.max(run.source_range.start);
    let source_end = byte_end.min(run.source_range.end);
    if source_start >= source_end {
        return 0.0;
    }
    debug_assert!(
        include_kerning || !run.include_kerning,
        "include_kerning=false requires an unkerned shaped run"
    );

    run.lines
        .iter()
        .map(|line| {
            measured_source_width_from_glyphs(
                &run.source_text,
                run.source_range.start,
                line.glyphs.as_slice(),
                source_start,
                source_end,
            )
        })
        .fold(0.0_f32, f32::max)
}

fn measured_grapheme_widths_from_shaped(shaped: &ShapedGlyphRun, text: &str) -> Vec<f32> {
    crate::profile_scope!("runtime", "text.measure", "grapheme_projection");
    let source_offset = shaped.source_range.start;
    let graphemes = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| {
            let end = start + grapheme.len();
            (source_offset + start, source_offset + end)
        })
        .collect::<Vec<_>>();
    let mut widths = vec![0.0; graphemes.len()];

    // Glyphs may be in visual order, so map each source interval directly instead of scanning the
    // complete shaped run once for every grapheme. A ligature can cover many graphemes; compute
    // its total span once and reuse it while projecting that glyph into source order.
    for glyph in shaped.lines.iter().flat_map(|line| &line.glyphs) {
        let first_overlapping =
            graphemes.partition_point(|&(_, end)| end <= glyph.source_range.start);
        let after_last_overlapping =
            graphemes.partition_point(|&(start, _)| start < glyph.source_range.end);
        if first_overlapping >= after_last_overlapping {
            continue;
        }

        let glyph_span =
            source_grapheme_span(&shaped.source_text, source_offset, glyph.source_range);
        for index in first_overlapping..after_last_overlapping {
            let (source_start, source_end) = graphemes[index];
            widths[index] += measured_glyph_source_overlap_with_span(
                &shaped.source_text,
                source_offset,
                glyph,
                source_start,
                source_end,
                glyph_span,
            );
        }
    }

    widths
}

pub(crate) fn line_metrics_with_provider<P>(style: &TextStyle, provider: &mut P) -> TextLineMetrics
where
    P: TextShapeRunProvider + ?Sized,
{
    let requested_line_height = resolved_line_height(style);
    let mut metrics = shape_line_with_provider(DEFAULT_METRICS_SAMPLE, style, provider);
    metrics.line_height = requested_line_height.max(metrics.line_height);
    metrics.baseline = metrics.baseline.clamp(0.0, metrics.line_height);
    metrics
}

fn shape_line_with_provider<P>(text: &str, style: &TextStyle, provider: &mut P) -> TextLineMetrics
where
    P: TextShapeRunProvider + ?Sized,
{
    let shaped = shape_unconstrained_line_with_provider(text, style, provider);
    shaped.lines.first().map_or(
        TextLineMetrics {
            width: 0.0,
            baseline: style.font_size.max(1.0) * 0.8,
            line_height: resolved_line_height(style),
        },
        |line| TextLineMetrics {
            width: line.measured_width,
            baseline: line.baseline,
            line_height: line.line_height,
        },
    )
}

#[cfg(test)]
fn shape_unconstrained_line(text: &str, style: &TextStyle) -> ShapedGlyphRun {
    shape_unconstrained_line_with_kerning(text, style, true)
}

fn shape_unconstrained_line_with_provider<P>(
    text: &str,
    style: &TextStyle,
    provider: &mut P,
) -> Arc<ShapedGlyphRun>
where
    P: TextShapeRunProvider + ?Sized,
{
    shape_unconstrained_line_with_kerning_and_provider(text, style, true, provider)
}

#[cfg(test)]
fn shape_unconstrained_line_with_kerning(
    text: &str,
    style: &TextStyle,
    include_kerning: bool,
) -> ShapedGlyphRun {
    let mut provider = DirectTextShapeRunProvider;
    (*shape_unconstrained_line_with_kerning_and_provider(
        text,
        style,
        include_kerning,
        &mut provider,
    ))
    .clone()
}

fn shape_unconstrained_line_with_kerning_and_provider<P>(
    text: &str,
    style: &TextStyle,
    include_kerning: bool,
    provider: &mut P,
) -> Arc<ShapedGlyphRun>
where
    P: TextShapeRunProvider + ?Sized,
{
    shape_horizontal_line_with_kerning_and_provider(
        text,
        style,
        TextDirection::Auto,
        TextRange {
            start: 0,
            end: text.len(),
        },
        include_kerning,
        provider,
    )
}

#[cfg(test)]
fn shape_horizontal_line(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
) -> ShapedGlyphRun {
    shape_horizontal_line_with_kerning(text, style, direction, source_range, true)
}

#[cfg(test)]
fn shape_horizontal_line_with_kerning(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    include_kerning: bool,
) -> ShapedGlyphRun {
    let mut provider = DirectTextShapeRunProvider;
    (*shape_horizontal_line_with_kerning_and_provider(
        text,
        style,
        direction,
        source_range,
        include_kerning,
        &mut provider,
    ))
    .clone()
}

fn shape_horizontal_line_with_kerning_and_provider<P>(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    include_kerning: bool,
    provider: &mut P,
) -> Arc<ShapedGlyphRun>
where
    P: TextShapeRunProvider + ?Sized,
{
    provider.shape_horizontal_line_with_kerning(
        text,
        style,
        direction,
        source_range,
        include_kerning,
    )
}

fn measured_source_width_from_glyphs(
    source_text: &str,
    source_offset: usize,
    glyphs: &[ShapedGlyph],
    source_start: usize,
    source_end: usize,
) -> f32 {
    glyphs
        .iter()
        .map(|glyph| {
            measured_glyph_source_overlap(
                source_text,
                source_offset,
                glyph,
                source_start,
                source_end,
            )
        })
        .sum()
}

fn measured_glyph_source_overlap(
    source_text: &str,
    source_offset: usize,
    glyph: &ShapedGlyph,
    source_start: usize,
    source_end: usize,
) -> f32 {
    let glyph_span = source_grapheme_span(source_text, source_offset, glyph.source_range);
    measured_glyph_source_overlap_with_span(
        source_text,
        source_offset,
        glyph,
        source_start,
        source_end,
        glyph_span,
    )
}

fn measured_glyph_source_overlap_with_span(
    source_text: &str,
    source_offset: usize,
    glyph: &ShapedGlyph,
    source_start: usize,
    source_end: usize,
    glyph_span: f32,
) -> f32 {
    let overlap_start = glyph.source_range.start.max(source_start);
    let overlap_end = glyph.source_range.end.min(source_end);
    if overlap_start >= overlap_end {
        return 0.0;
    }

    let advance = glyph.advance.max(0.0);
    if overlap_start == glyph.source_range.start && overlap_end == glyph.source_range.end {
        return advance;
    }

    let overlap_span = source_grapheme_span(
        source_text,
        source_offset,
        TextRange {
            start: overlap_start,
            end: overlap_end,
        },
    );
    if glyph_span <= 0.0 || overlap_span <= 0.0 {
        return advance;
    }
    advance * (overlap_span / glyph_span).clamp(0.0, 1.0)
}

fn source_grapheme_span(source_text: &str, source_offset: usize, range: TextRange) -> f32 {
    let Some(start) = range.start.checked_sub(source_offset) else {
        return 1.0;
    };
    let Some(end) = range.end.checked_sub(source_offset) else {
        return 1.0;
    };
    let start = start.min(source_text.len());
    let end = end.min(source_text.len()).max(start);
    if !source_text.is_char_boundary(start) || !source_text.is_char_boundary(end) {
        return 1.0;
    }
    source_text[start..end].graphemes(true).count().max(1) as f32
}

fn resolved_line_height(style: &TextStyle) -> f32 {
    style.line_height.max(style.font_size.max(1.0))
}

#[cfg(test)]
mod tests;
