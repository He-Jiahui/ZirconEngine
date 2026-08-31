use crate::text::{HorizontalGlyphMetricSpan, HorizontalLineRawMetrics, ShapedHardLine, TextStyle};

use super::TextLineMetrics;

/// Resolves the Plain horizontal line box from direct-shaping metric provenance.
///
/// Every fallback face in the current Plain path shares one alphabetic baseline. The artifact
/// renderers consume that baseline directly, then add each glyph's shaping offset. Therefore a
/// per-face block-top offset must not be copied into `TextGlyph::offset[1]`: doing so would apply
/// the ascent correction twice. Rich text and inline objects use separate block origins and join
/// a later policy adapter instead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HorizontalPlainLinePolicy {
    pub(crate) metrics: TextLineMetrics,
}

pub(crate) fn resolve_horizontal_plain_line_policy(
    style: &TextStyle,
    line: &ShapedHardLine,
    raw_metrics: HorizontalLineRawMetrics,
    spans: &[HorizontalGlyphMetricSpan],
) -> Option<HorizontalPlainLinePolicy> {
    spans_match_line_envelope(line, raw_metrics, spans)?;
    let requested_line_height = style.line_height.max(style.font_size.max(1.0));
    if !requested_line_height.is_finite()
        || !line.measured_width.is_finite()
        || line.measured_width < 0.0
    {
        return None;
    }
    let content_height = raw_metrics.ascent() + raw_metrics.descent();
    let natural_line_height = content_height + raw_metrics.line_spacing_gap();
    if !content_height.is_finite() || !natural_line_height.is_finite() {
        return None;
    }
    let line_height = requested_line_height.max(natural_line_height);
    let baseline = (line_height - content_height).max(0.0) * 0.5 + raw_metrics.ascent();
    (line_height.is_finite() && baseline.is_finite()).then_some(HorizontalPlainLinePolicy {
        metrics: TextLineMetrics {
            width: line.measured_width,
            baseline,
            line_height,
        },
    })
}

fn spans_match_line_envelope(
    line: &ShapedHardLine,
    raw_metrics: HorizontalLineRawMetrics,
    spans: &[HorizontalGlyphMetricSpan],
) -> Option<()> {
    if line.glyphs.is_empty() || spans.is_empty() {
        return None;
    }
    let mut expected_start = 0_usize;
    let mut max_ascent = 0.0_f32;
    let mut max_descent = 0.0_f32;
    for span in spans {
        if span.glyph_start != expected_start
            || span.glyph_start >= span.glyph_end
            || span.glyph_end > line.glyphs.len()
        {
            return None;
        }
        expected_start = span.glyph_end;
        max_ascent = max_ascent.max(span.metrics.ascent());
        max_descent = max_descent.max(span.metrics.descent());
    }
    (expected_start == line.glyphs.len()
        && max_ascent == raw_metrics.ascent()
        && max_descent == raw_metrics.descent())
    .then_some(())
}

#[cfg(test)]
mod tests {
    use crate::core::framework::text::TextDirection;
    use crate::text::{
        HorizontalGlyphMetricSpan, HorizontalLineRawMetrics, ShapedGlyph, ShapedHardLine,
        TextRange, TextStyle,
    };

    use super::resolve_horizontal_plain_line_policy;

    #[test]
    fn plain_fallback_faces_share_the_composite_alphabetic_baseline_without_offsets() {
        let primary = HorizontalLineRawMetrics::new(12.0, 4.0, 2.0).expect("valid metrics");
        let fallback = HorizontalLineRawMetrics::new(15.0, 6.0, 0.0).expect("valid metrics");
        let line = test_line(2);
        let policy = resolve_horizontal_plain_line_policy(
            &TextStyle {
                font_size: 16.0,
                line_height: 18.0,
                ..TextStyle::default()
            },
            &line,
            HorizontalLineRawMetrics::new(15.0, 6.0, 2.0).expect("valid envelope"),
            &[
                HorizontalGlyphMetricSpan {
                    line_index: 0,
                    glyph_start: 0,
                    glyph_end: 1,
                    metrics: primary,
                },
                HorizontalGlyphMetricSpan {
                    line_index: 0,
                    glyph_start: 1,
                    glyph_end: 2,
                    metrics: fallback,
                },
            ],
        )
        .expect("complete selected-face provenance");

        assert_eq!(policy.metrics.width, 20.0);
        assert_eq!(policy.metrics.baseline, 15.0);
        assert_eq!(policy.metrics.line_height, 23.0);
    }

    #[test]
    fn partial_selected_face_provenance_cannot_replace_existing_line_metrics() {
        let metrics = HorizontalLineRawMetrics::new(12.0, 4.0, 2.0).expect("valid metrics");
        let line = test_line(2);

        assert!(
            resolve_horizontal_plain_line_policy(
                &TextStyle::default(),
                &line,
                metrics,
                &[HorizontalGlyphMetricSpan {
                    line_index: 0,
                    glyph_start: 0,
                    glyph_end: 1,
                    metrics,
                }],
            )
            .is_none()
        );
    }

    fn test_line(glyph_count: usize) -> ShapedHardLine {
        ShapedHardLine {
            line_index: 0,
            source_range: TextRange {
                start: 0,
                end: glyph_count,
            },
            visual_range: TextRange {
                start: 0,
                end: glyph_count,
            },
            measured_width: 20.0,
            baseline: 0.0,
            line_height: 0.0,
            glyphs: (0..glyph_count)
                .map(|index| ShapedGlyph {
                    glyph_id: 1,
                    font_id: None,
                    font_instance_id: None,
                    source_range: TextRange {
                        start: index,
                        end: index + 1,
                    },
                    visual_range: TextRange {
                        start: index,
                        end: index + 1,
                    },
                    advance: 10.0,
                    x: index as f32 * 10.0,
                    y: 0.0,
                    offset_x: 0.0,
                    offset_y: 0.0,
                    direction: TextDirection::LeftToRight,
                    bidi_level: 0,
                    cluster_flags: Default::default(),
                    rotation: Default::default(),
                    script: Default::default(),
                })
                .collect(),
        }
    }
}
