use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{layout::UiSize, surface::UiResolvedStyle};

use crate::core::framework::render::{ShapedGlyph, ShapedGlyphRun};
use crate::graphics::text::shaping::shape_horizontal_line_with_kerning as shape_backend_horizontal_line_with_kerning;
use zircon_runtime_interface::ui::surface::{UiTextDirection, UiTextRange};

use super::tab::tab_aligned_width;

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

    if !text.contains('\t') {
        return shape_line(text, style).width;
    }

    let grapheme_widths = measured_grapheme_widths(text, style);
    tab_aligned_width(text, &grapheme_widths, style, shape_line(" ", style).width)
}

pub(crate) fn measured_grapheme_widths(text: &str, style: &UiResolvedStyle) -> Vec<f32> {
    let shaped = shape_unconstrained_line(text, style);
    text.grapheme_indices(true)
        .map(|(start, grapheme)| {
            let end = start + grapheme.len();
            measured_width(&shaped, start, end, true)
        })
        .collect()
}

pub(crate) fn measure_text_source_range_width(
    text: &str,
    style: &UiResolvedStyle,
    range: UiTextRange,
) -> f32 {
    measure_text_source_range_width_with_kerning(text, style, range, true)
}

pub(crate) fn measure_text_source_range_width_with_kerning(
    text: &str,
    style: &UiResolvedStyle,
    range: UiTextRange,
    include_kerning: bool,
) -> f32 {
    if text.is_empty() || range.start >= range.end {
        return 0.0;
    }

    let shaped = shape_unconstrained_line_with_kerning(text, style, include_kerning);
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
    shape_unconstrained_line_with_kerning(text, style, true)
}

fn shape_unconstrained_line_with_kerning(
    text: &str,
    style: &UiResolvedStyle,
    include_kerning: bool,
) -> ShapedGlyphRun {
    shape_horizontal_line_with_kerning(
        text,
        style,
        UiTextDirection::Auto,
        UiTextRange {
            start: 0,
            end: text.len(),
        },
        include_kerning,
    )
}

fn shape_horizontal_line(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
) -> ShapedGlyphRun {
    shape_horizontal_line_with_kerning(text, style, direction, source_range, true)
}

fn shape_horizontal_line_with_kerning(
    text: &str,
    style: &UiResolvedStyle,
    direction: UiTextDirection,
    source_range: UiTextRange,
    include_kerning: bool,
) -> ShapedGlyphRun {
    shape_backend_horizontal_line_with_kerning(
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
    let overlap_start = glyph.source_range.start.max(source_start);
    let overlap_end = glyph.source_range.end.min(source_end);
    if overlap_start >= overlap_end {
        return 0.0;
    }

    let advance = glyph.advance.max(0.0);
    if overlap_start == glyph.source_range.start && overlap_end == glyph.source_range.end {
        return advance;
    }

    let glyph_span = source_grapheme_span(source_text, source_offset, glyph.source_range);
    let overlap_span = source_grapheme_span(
        source_text,
        source_offset,
        UiTextRange {
            start: overlap_start,
            end: overlap_end,
        },
    );
    if glyph_span <= 0.0 || overlap_span <= 0.0 {
        return advance;
    }
    advance * (overlap_span / glyph_span).clamp(0.0, 1.0)
}

fn source_grapheme_span(source_text: &str, source_offset: usize, range: UiTextRange) -> f32 {
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

fn resolved_line_height(style: &UiResolvedStyle) -> f32 {
    style.line_height.max(style.font_size.max(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_width_sums_source_subranges() {
        let style = test_style();
        let shaped = shape_unconstrained_line("Wi", &style);
        let line = shaped.lines.first().expect("shaped line");
        let first = measured_width(&shaped, 0, 1, true);
        let second = measured_width(&shaped, 1, 2, true);
        let full = measured_width(&shaped, 0, 2, true);

        assert!(first > 0.0);
        assert!(second > 0.0);
        assert!((first + second - full).abs() < 0.1);
        assert!((full - line.measured_width).abs() < 0.1);
    }

    #[test]
    fn measured_width_uses_absolute_source_ranges() {
        let style = test_style();
        let source = "xxWi";
        let shaped = shape_horizontal_line(
            &source[2..],
            &style,
            UiTextDirection::LeftToRight,
            UiTextRange {
                start: 2,
                end: source.len(),
            },
        );
        let local_shaped = shape_unconstrained_line("Wi", &style);

        assert_eq!(measured_width(&shaped, 0, 2, true), 0.0);
        assert!(
            (measured_width(&shaped, 2, 3, true) - measured_width(&local_shaped, 0, 1, true)).abs()
                < 0.1
        );
        assert!(
            (measured_width(&shaped, 2, source.len(), true)
                - measured_width(&local_shaped, 0, 2, true))
            .abs()
                < 0.1
        );
    }

    #[test]
    fn measured_width_splits_partial_cluster_by_grapheme_count() {
        let style = test_style();
        let shaped = shape_unconstrained_line("fi", &style);
        let first = measured_width(&shaped, 0, 1, true);
        let second = measured_width(&shaped, 1, 2, true);
        let full = measured_width(&shaped, 0, 2, true);

        assert!(first > 0.0);
        assert!(second > 0.0);
        assert!((first + second - full).abs() < 0.1);
    }

    #[test]
    fn measure_source_range_can_request_unkerned_backend_shape() {
        let style = test_style();
        let range = UiTextRange { start: 0, end: 2 };
        let kerned = shape_unconstrained_line_with_kerning("AV", &style, true);
        let unkerned = shape_unconstrained_line_with_kerning("AV", &style, false);
        let kerned_width = measure_text_source_range_width_with_kerning("AV", &style, range, true);
        let unkerned_width =
            measure_text_source_range_width_with_kerning("AV", &style, range, false);

        assert!(kerned.include_kerning);
        assert!(!unkerned.include_kerning);
        assert!((kerned_width - measured_width(&kerned, 0, 2, true)).abs() < 0.1);
        assert!((unkerned_width - measured_width(&unkerned, 0, 2, false)).abs() < 0.1);
    }

    fn test_style() -> UiResolvedStyle {
        UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        }
    }
}
