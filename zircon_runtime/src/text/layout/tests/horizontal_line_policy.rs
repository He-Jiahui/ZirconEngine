use crate::core::framework::text::TextDirection;
use crate::text::{
    HorizontalGlyphMetricSpan, HorizontalLineRawMetrics, ShapedGlyph, ShapedHardLine, TextRange,
    TextStyle,
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

    assert!(resolve_horizontal_plain_line_policy(
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
    .is_none());
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
