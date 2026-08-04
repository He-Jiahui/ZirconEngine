use super::*;
use std::sync::Arc;

use crate::{
    core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation},
    text::{
        register_resolved_text_glyph_artifact, ResolvedTextGlyphArtifact,
        ResolvedTextGlyphArtifactLine,
    },
};
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextRun, UiTextDirection, UiTextRunKind, UiTextWritingMode,
};

#[test]
fn source_geometry_uses_resolved_glyph_advances() {
    let layout = layout_with_advances("a\tb", vec![6.0, 18.0, 6.0]);

    let caret = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 2,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("caret frame");
    let frames = text_range_frames_for_text_layout(&layout, UiTextRange { start: 1, end: 3 });

    assert_eq!(caret, UiFrame::new(34.0, 20.0, 1.0, 12.0));
    assert_eq!(frames, vec![UiFrame::new(16.0, 20.0, 24.0, 12.0)]);
}

#[test]
fn source_geometry_prefers_artifact_cluster_edges_over_dto_grapheme_advances() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let mut layout = layout_with_advances("fi", vec![12.0, 18.0]);
    let line = layout.lines[0].clone();
    layout.rich_text_artifact = Some(register_resolved_text_glyph_artifact(Arc::new(
        ResolvedTextGlyphArtifact {
            source_text: Arc::from("fi"),
            source_text_origin: 0,
            font_generation: crate::text::font::shared_font_database_generation(),
            style: UiResolvedStyle::default(),
            writing_mode: UiTextWritingMode::HorizontalTb,
            lines: vec![Some(ResolvedTextGlyphArtifactLine {
                glyphs: vec![TextGlyph {
                    glyph_id: 77,
                    source_range: 0..2,
                    visual_range: 0..2,
                    advance: 30.0,
                    position: [0.0, 0.0],
                    offset: [0.0, 0.0],
                    font_face: None,
                    font_instance: None,
                    rotation: TextGlyphRotation::None,
                    bidi_level: 0,
                    flags: TextGlyphFlags {
                        cluster_start: true,
                        ..TextGlyphFlags::default()
                    },
                    requires_rasterization: true,
                }],
                layout_line: line,
            })],
        },
    )));

    let upstream = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Upstream,
        },
    )
    .expect("upstream cluster edge");
    let downstream = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("downstream cluster edge");
    let frames = text_range_frames_for_text_layout(&layout, UiTextRange { start: 0, end: 1 });

    assert_eq!(upstream, UiFrame::new(10.0, 20.0, 1.0, 12.0));
    assert_eq!(downstream, UiFrame::new(40.0, 20.0, 1.0, 12.0));
    assert_eq!(frames, vec![UiFrame::new(10.0, 20.0, 30.0, 12.0)]);
}

#[test]
fn source_geometry_uses_vertical_writing_mode_advances() {
    let mut layout = layout_with_advances("abc", vec![6.0, 18.0, 6.0]);
    layout.writing_mode = zircon_runtime_interface::ui::surface::UiTextWritingMode::VerticalRl;
    let line = layout.lines.first_mut().expect("line");
    line.frame = UiFrame::new(20.0, 10.0, 10.0, 30.0);
    line.measured_width = 30.0;

    let caret = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 2,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("caret frame");
    let frames = text_range_frames_for_text_layout(&layout, UiTextRange { start: 1, end: 3 });

    assert_eq!(caret, UiFrame::new(20.0, 34.0, 10.0, 1.0));
    assert_eq!(frames, vec![UiFrame::new(20.0, 16.0, 10.0, 24.0)]);
}

#[test]
fn vertical_source_geometry_uses_artifact_clusters_and_falls_back_when_stale_or_mismatched() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let mut layout = layout_with_advances("fi", vec![12.0, 18.0]);
    layout.writing_mode = UiTextWritingMode::VerticalRl;
    let line = layout.lines.first_mut().expect("line");
    line.frame = UiFrame::new(20.0, 10.0, 10.0, 30.0);
    line.measured_width = 30.0;
    let artifact_line = line.clone();
    let glyphs = vec![TextGlyph {
        glyph_id: 77,
        source_range: 0..2,
        visual_range: 0..2,
        advance: 30.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags {
            cluster_start: true,
            ..TextGlyphFlags::default()
        },
        requires_rasterization: true,
    }];
    let attach_artifact = |font_generation| {
        register_resolved_text_glyph_artifact(Arc::new(ResolvedTextGlyphArtifact {
            source_text: Arc::from("fi"),
            source_text_origin: 0,
            font_generation,
            style: UiResolvedStyle::default(),
            writing_mode: UiTextWritingMode::VerticalRl,
            lines: vec![Some(ResolvedTextGlyphArtifactLine {
                glyphs: glyphs.clone(),
                layout_line: artifact_line.clone(),
            })],
        }))
    };
    layout.rich_text_artifact = Some(attach_artifact(
        crate::text::font::shared_font_database_generation(),
    ));

    let artifact_caret = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("artifact caret");
    let artifact_selection =
        text_range_frames_for_text_layout(&layout, UiTextRange { start: 0, end: 1 });

    assert_eq!(artifact_caret, UiFrame::new(20.0, 40.0, 10.0, 1.0));
    assert_eq!(
        artifact_selection,
        vec![UiFrame::new(20.0, 10.0, 10.0, 30.0)]
    );

    layout.writing_mode = UiTextWritingMode::HorizontalTb;
    let writing_mode_mismatched_caret = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("writing mode mismatched DTO caret");
    let writing_mode_mismatched_selection =
        text_range_frames_for_text_layout(&layout, UiTextRange { start: 0, end: 1 });

    assert_eq!(
        writing_mode_mismatched_caret,
        UiFrame::new(32.0, 10.0, 1.0, 30.0)
    );
    assert_eq!(
        writing_mode_mismatched_selection,
        vec![UiFrame::new(20.0, 10.0, 12.0, 30.0)]
    );

    layout.writing_mode = UiTextWritingMode::VerticalRl;
    layout.lines[0].glyph_advances[0] = 13.0;
    let mismatched_caret = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("mismatched DTO caret");
    let mismatched_selection =
        text_range_frames_for_text_layout(&layout, UiTextRange { start: 0, end: 1 });

    assert_eq!(mismatched_caret, UiFrame::new(20.0, 23.0, 10.0, 1.0));
    assert_eq!(
        mismatched_selection,
        vec![UiFrame::new(20.0, 10.0, 10.0, 13.0)]
    );

    layout.lines[0].glyph_advances[0] = 12.0;
    layout.rich_text_artifact = Some(attach_artifact(
        crate::text::font::shared_font_database_generation().wrapping_add(1),
    ));
    let stale_caret = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("stale DTO caret");
    let stale_selection =
        text_range_frames_for_text_layout(&layout, UiTextRange { start: 0, end: 1 });

    assert_eq!(stale_caret, UiFrame::new(20.0, 22.0, 10.0, 1.0));
    assert_eq!(stale_selection, vec![UiFrame::new(20.0, 10.0, 10.0, 12.0)]);
}

#[test]
fn text_caret_affinity_soft_wrap_boundary() {
    let mut layout = layout_with_advances("ab", vec![6.0, 6.0]);
    let mut second_line = layout.lines[0].clone();
    second_line.text = "cd".to_string();
    second_line.frame = UiFrame::new(10.0, 32.0, 12.0, 12.0);
    second_line.source_range = UiTextRange { start: 2, end: 4 };
    second_line.visual_range = UiTextRange { start: 0, end: 2 };
    second_line.runs[0].text = second_line.text.clone();
    second_line.runs[0].source_range = second_line.source_range;
    second_line.runs[0].visual_range = second_line.visual_range;
    layout.lines.push(second_line);
    layout.source_range = UiTextRange { start: 0, end: 4 };

    let upstream = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 2,
            affinity: UiTextCaretAffinity::Upstream,
        },
    )
    .expect("upstream caret frame");
    let downstream = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 2,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("downstream caret frame");

    assert_eq!(upstream, UiFrame::new(22.0, 20.0, 1.0, 12.0));
    assert_eq!(downstream, UiFrame::new(10.0, 32.0, 1.0, 12.0));
}

#[test]
fn source_geometry_with_source_metrics_uses_shaped_source_range_width() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = "Wi";
    let layout = layout_with_advances(text, vec![1.0, 1.0]);
    let caret = UiTextCaret {
        offset: "W".len(),
        affinity: UiTextCaretAffinity::Downstream,
    };
    let shaped_prefix = measure_text_source_range_width(
        text,
        &style,
        UiTextRange {
            start: 0,
            end: "W".len(),
        },
    );

    let stale = caret_frame_for_text_layout(&layout, &caret).expect("stale caret");
    let measured = caret_frame_for_text_layout_with_source_metrics(&layout, &caret, text, &style)
        .expect("measured caret");

    assert_eq!(stale, UiFrame::new(11.0, 20.0, 1.0, 12.0));
    assert!((measured.x - (10.0 + shaped_prefix)).abs() < 0.1);
    assert!(
        (measured.x - stale.x).abs() > 0.5,
        "source metrics should not reuse stale per-grapheme advances"
    );
}

#[test]
fn source_geometry_with_source_metrics_keeps_tab_aligned_advances() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = "a\tb";
    let layout = layout_with_advances(text, vec![6.0, 18.0, 6.0]);
    let caret = UiTextCaret {
        offset: 2,
        affinity: UiTextCaretAffinity::Downstream,
    };

    let measured = caret_frame_for_text_layout_with_source_metrics(&layout, &caret, text, &style)
        .expect("measured caret");

    assert_eq!(measured, UiFrame::new(34.0, 20.0, 1.0, 12.0));
}

#[test]
fn source_geometry_with_source_metrics_keeps_vertical_advances() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = "Wi";
    let mut layout = layout_with_advances(text, vec![2.0, 20.0]);
    layout.writing_mode = zircon_runtime_interface::ui::surface::UiTextWritingMode::VerticalRl;
    let line = layout.lines.first_mut().expect("line");
    line.frame = UiFrame::new(20.0, 10.0, 10.0, 30.0);
    let caret = UiTextCaret {
        offset: "W".len(),
        affinity: UiTextCaretAffinity::Downstream,
    };
    let horizontal_source_width = measure_text_source_range_width(
        text,
        &style,
        UiTextRange {
            start: 0,
            end: "W".len(),
        },
    );

    let fallback = caret_frame_for_text_layout(&layout, &caret).expect("fallback caret");
    let measured = caret_frame_for_text_layout_with_source_metrics(&layout, &caret, text, &style)
        .expect("measured caret");

    assert_eq!(fallback, UiFrame::new(20.0, 12.0, 10.0, 1.0));
    assert_eq!(measured, fallback);
    assert!(
        (horizontal_source_width - 2.0).abs() > 0.5,
        "test must prove the vertical path did not consume horizontal source width"
    );
}

#[test]
fn source_geometry_with_source_metrics_rejects_unresolved_auto_direction() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = "Wi";
    let mut layout = layout_with_advances(text, vec![2.0, 20.0]);
    layout.direction = UiTextDirection::Auto;
    let caret = UiTextCaret {
        offset: "W".len(),
        affinity: UiTextCaretAffinity::Downstream,
    };
    let horizontal_source_width = measure_text_source_range_width(
        text,
        &style,
        UiTextRange {
            start: 0,
            end: "W".len(),
        },
    );

    let fallback = caret_frame_for_text_layout(&layout, &caret).expect("fallback caret");
    let measured = caret_frame_for_text_layout_with_source_metrics(&layout, &caret, text, &style)
        .expect("measured caret");

    assert_eq!(fallback, UiFrame::new(12.0, 20.0, 1.0, 12.0));
    assert_eq!(measured, fallback);
    assert!(!line_accepts_source_measure(
        &layout,
        layout.lines.first().expect("line"),
        text
    ));
    assert!(
        (horizontal_source_width - 2.0).abs() > 0.5,
        "test must prove unresolved Auto did not consume horizontal source width"
    );
}

#[test]
fn source_geometry_with_source_metrics_requires_ltr_line_and_run_direction() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = "Wi";
    let mut layout = layout_with_advances(text, vec![2.0, 20.0]);
    assert!(line_accepts_source_measure(
        &layout,
        layout.lines.first().expect("line"),
        text
    ));

    layout.lines[0].direction = UiTextDirection::Auto;
    assert!(!line_accepts_source_measure(
        &layout,
        layout.lines.first().expect("line"),
        text
    ));

    layout.lines[0].direction = UiTextDirection::LeftToRight;
    layout.lines[0].runs[0].direction = UiTextDirection::Auto;
    assert!(!line_accepts_source_measure(
        &layout,
        layout.lines.first().expect("line"),
        text
    ));

    let measured = caret_frame_for_text_layout_with_source_metrics(
        &layout,
        &UiTextCaret {
            offset: "W".len(),
            affinity: UiTextCaretAffinity::Downstream,
        },
        text,
        &style,
    )
    .expect("measured caret");

    assert_eq!(measured, UiFrame::new(12.0, 20.0, 1.0, 12.0));
}

#[test]
fn source_geometry_with_source_metrics_uses_absolute_source_prefix_ranges() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let source = "sample base.zui";
    let line_text = "base.zui";
    let line_start = "sample ".len();
    let caret_offset = line_start + "base".len();
    let mut layout = layout_with_advances(line_text, vec![1.0; line_text.len()]);
    {
        let line = layout.lines.first_mut().expect("line");
        line.source_range = UiTextRange {
            start: line_start,
            end: source.len(),
        };
        line.runs[0].source_range = line.source_range;
    }

    let measured = caret_frame_for_text_layout_with_source_metrics(
        &layout,
        &UiTextCaret {
            offset: caret_offset,
            affinity: UiTextCaretAffinity::Downstream,
        },
        source,
        &style,
    )
    .expect("measured caret");
    let expected_prefix = UiTextRange {
        start: line_start,
        end: caret_offset,
    };
    let expected_width = measure_text_source_range_width(source, &style, expected_prefix);
    let line = layout.lines.first().expect("line");

    assert_eq!(
        source_prefix_range_for_visual_offset(line, "base".len()),
        expected_prefix
    );
    assert!((measured.x - (10.0 + expected_width)).abs() < 0.1);
}

fn layout_with_advances(text: &str, glyph_advances: Vec<f32>) -> UiResolvedTextLayout {
    UiResolvedTextLayout {
        font_size: 10.0,
        line_height: 12.0,
        source_range: UiTextRange {
            start: 0,
            end: text.len(),
        },
        direction: UiTextDirection::LeftToRight,
        lines: vec![UiResolvedTextLine {
            text: text.to_string(),
            frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
            source_range: UiTextRange {
                start: 0,
                end: text.len(),
            },
            visual_range: UiTextRange {
                start: 0,
                end: text.len(),
            },
            measured_width: 30.0,
            glyph_advances,
            baseline: 9.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: text.to_string(),
                source_range: UiTextRange {
                    start: 0,
                    end: text.len(),
                },
                visual_range: UiTextRange {
                    start: 0,
                    end: text.len(),
                },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    }
}
