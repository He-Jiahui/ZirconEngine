use std::sync::Arc;

use crate::text::{TextSize, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::TextRange;
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
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
            TextDirection::LeftToRight,
            TextRange {
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
    fn measured_grapheme_widths_projects_visual_glyphs_into_source_order() {
        let shaped = ShapedGlyphRun {
            source_text: std::sync::Arc::from("abc"),
            source_range: TextRange { start: 0, end: 3 },
            direction: TextDirection::LeftToRight,
            orientation: crate::text::TextOrientation::Horizontal,
            vertical_mode: crate::text::VerticalMode::Mixed,
            include_kerning: true,
            measured_width: 50.0,
            measured_height: 12.0,
            lines: vec![crate::text::ShapedTextLine {
                line_index: 0,
                source_range: TextRange { start: 0, end: 3 },
                visual_range: TextRange { start: 0, end: 3 },
                measured_width: 50.0,
                baseline: 9.0,
                line_height: 12.0,
                glyphs: vec![test_glyph(2, 3, 30.0), test_glyph(0, 2, 20.0)],
            }],
        };

        assert_eq!(
            measured_grapheme_widths_from_shaped(&shaped, "abc"),
            vec![10.0, 10.0, 30.0]
        );
    }

    #[test]
    fn measured_grapheme_widths_distributes_a_multi_grapheme_cluster_once_per_glyph() {
        let shaped = ShapedGlyphRun {
            source_text: std::sync::Arc::from("abcd"),
            source_range: TextRange { start: 0, end: 4 },
            direction: TextDirection::LeftToRight,
            orientation: crate::text::TextOrientation::Horizontal,
            vertical_mode: crate::text::VerticalMode::Mixed,
            include_kerning: true,
            measured_width: 40.0,
            measured_height: 12.0,
            lines: vec![crate::text::ShapedTextLine {
                line_index: 0,
                source_range: TextRange { start: 0, end: 4 },
                visual_range: TextRange { start: 0, end: 4 },
                measured_width: 40.0,
                baseline: 9.0,
                line_height: 12.0,
                glyphs: vec![test_glyph(0, 4, 40.0)],
            }],
        };

        assert_eq!(
            measured_grapheme_widths_from_shaped(&shaped, "abcd"),
            vec![10.0, 10.0, 10.0, 10.0]
        );
    }

    #[test]
    fn measured_grapheme_widths_shapes_the_complete_text_once() {
        let shaped = Arc::new(ShapedGlyphRun {
            source_text: std::sync::Arc::from("abcd"),
            source_range: TextRange { start: 0, end: 4 },
            direction: TextDirection::LeftToRight,
            orientation: crate::text::TextOrientation::Horizontal,
            vertical_mode: crate::text::VerticalMode::Mixed,
            include_kerning: true,
            measured_width: 40.0,
            measured_height: 12.0,
            lines: vec![crate::text::ShapedTextLine {
                line_index: 0,
                source_range: TextRange { start: 0, end: 4 },
                visual_range: TextRange { start: 0, end: 4 },
                measured_width: 40.0,
                baseline: 9.0,
                line_height: 12.0,
                glyphs: vec![test_glyph(0, 4, 40.0)],
            }],
        });
        let mut provider = CountingShapeRunProvider {
            shaped,
            shape_calls: 0,
        };

        assert_eq!(
            measured_grapheme_widths_with_provider("abcd", &test_style(), &mut provider),
            vec![10.0, 10.0, 10.0, 10.0]
        );
        assert_eq!(provider.shape_calls, 1);
    }

    #[test]
    fn measure_text_size_preserves_a_trailing_empty_line() {
        let style = test_style();
        let line_metrics = line_metrics_with_provider(&style, &mut DirectTextShapeRunProvider);
        let measured = measure_text_size("line\n", &style);

        assert!((measured.height - line_metrics.line_height * 2.0).abs() < 0.1);
    }

    #[test]
    fn measure_text_size_preserves_consecutive_trailing_empty_lines() {
        let style = test_style();
        let line_metrics = line_metrics_with_provider(&style, &mut DirectTextShapeRunProvider);
        let measured = measure_text_size("line\n\n", &style);

        assert!((measured.height - line_metrics.line_height * 3.0).abs() < 0.1);
    }

    #[test]
    fn measure_text_size_uses_all_mandatory_unicode_separators() {
        let style = test_style();
        let line_metrics = line_metrics_with_provider(&style, &mut DirectTextShapeRunProvider);
        let measured = measure_text_size("line\r\n\u{2028}", &style);

        assert!((measured.height - line_metrics.line_height * 3.0).abs() < 0.1);
        assert!((measured.width - measure_line_width("line", &style)).abs() < 0.1);
    }

    #[test]
    fn measure_source_range_can_request_unkerned_backend_shape() {
        let style = test_style();
        let range = TextRange { start: 0, end: 2 };
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

    fn test_style() -> TextStyle {
        TextStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..TextStyle::default()
        }
    }

    fn test_glyph(source_start: usize, source_end: usize, advance: f32) -> ShapedGlyph {
        ShapedGlyph {
            glyph_id: 0,
            font_id: None,
            font_instance_id: None,
            source_range: TextRange {
                start: source_start,
                end: source_end,
            },
            visual_range: TextRange {
                start: source_start,
                end: source_end,
            },
            advance,
            x: 0.0,
            y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            direction: TextDirection::LeftToRight,
            bidi_level: 0,
            cluster_flags: crate::text::ShapedGlyphClusterFlags::default(),
            rotation: crate::text::ShapedGlyphRotation::None,
            script: crate::text::ShapedGlyphScript::default(),
        }
    }

    struct CountingShapeRunProvider {
        shaped: Arc<ShapedGlyphRun>,
        shape_calls: usize,
    }

    impl TextShapeRunProvider for CountingShapeRunProvider {
        fn shape_horizontal_line_with_kerning(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _direction: TextDirection,
            _source_range: TextRange,
            _include_kerning: bool,
        ) -> Arc<ShapedGlyphRun> {
            self.shape_calls += 1;
            Arc::clone(&self.shaped)
        }
    }
}
