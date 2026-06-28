use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{layout::UiSize, surface::UiResolvedStyle};

use crate::core::framework::render::{ShapedGlyph, ShapedGlyphRun};
use crate::graphics::text::shaping::shape_horizontal_line;
use zircon_runtime_interface::ui::surface::{UiTextDirection, UiTextRange};

const DEFAULT_METRICS_SAMPLE: &str = "Hg";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TextLineMetrics {
    pub width: f32,
    pub baseline: f32,
    pub line_height: f32,
}

pub(crate) fn measure_text_size(text: &str, style: &UiResolvedStyle) -> UiSize {
    let metrics = line_metrics(style);
    let width = text
        .lines()
        .map(|line| measure_line_width(line, style))
        .fold(0.0_f32, f32::max);
    let line_count = text.lines().count().max(1) as f32;
    UiSize::new(width, metrics.line_height * line_count)
}

pub(crate) fn measure_line_width(text: &str, style: &UiResolvedStyle) -> f32 {
    if text.is_empty() {
        return 0.0;
    }

    shape_line(text, style).width
}

pub(crate) fn measured_grapheme_widths(text: &str, style: &UiResolvedStyle) -> Vec<f32> {
    let shaped = shape_unconstrained_line(text, style);
    let Some(line) = shaped.lines.first() else {
        return Vec::new();
    };
    text.grapheme_indices(true)
        .map(|(start, grapheme)| {
            let end = start + grapheme.len();
            measured_width_from_glyphs(line.text.as_str(), line.glyphs.as_slice(), start, end)
        })
        .collect()
}

pub(crate) fn line_metrics(style: &UiResolvedStyle) -> TextLineMetrics {
    let requested_line_height = resolved_line_height(style);
    let mut metrics = shape_line(DEFAULT_METRICS_SAMPLE, style);
    metrics.line_height = requested_line_height.max(metrics.line_height);
    metrics.baseline = metrics.baseline.clamp(0.0, metrics.line_height);
    metrics
}

fn shape_line(text: &str, style: &UiResolvedStyle) -> TextLineMetrics {
    let shaped = shape_unconstrained_line(text, style);
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

fn shape_unconstrained_line(text: &str, style: &UiResolvedStyle) -> ShapedGlyphRun {
    shape_horizontal_line(
        text,
        style,
        UiTextDirection::Auto,
        UiTextRange {
            start: 0,
            end: text.len(),
        },
    )
}

fn measured_width_from_glyphs(
    line_text: &str,
    glyphs: &[ShapedGlyph],
    visual_start: usize,
    visual_end: usize,
) -> f32 {
    glyphs
        .iter()
        .filter(|glyph| {
            glyph.visual_range.start < visual_end && glyph.visual_range.end > visual_start
        })
        .map(|glyph| glyph.advance.max(0.0) / glyph_grapheme_span(line_text, glyph))
        .sum()
}

fn glyph_grapheme_span(line_text: &str, glyph: &ShapedGlyph) -> f32 {
    let start = glyph.visual_range.start.min(line_text.len());
    let end = glyph.visual_range.end.min(line_text.len()).max(start);
    if !line_text.is_char_boundary(start) || !line_text.is_char_boundary(end) {
        return 1.0;
    }
    line_text[start..end].graphemes(true).count().max(1) as f32
}

fn resolved_line_height(style: &UiResolvedStyle) -> f32 {
    style.line_height.max(style.font_size.max(1.0))
}
