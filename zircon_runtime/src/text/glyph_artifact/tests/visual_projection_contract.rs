use super::*;

#[test]
fn virtual_source_run_uses_visual_fallback_when_artifact_is_unavailable() {
    let line = UiResolvedTextLine {
        text: "ـ".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 12.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 12.0,
        glyph_advances: vec![12.0],
        baseline: 9.0,
        direction: UiTextDirection::RightToLeft,
        runs: vec![visual_run("ـ", 2, 2, 0, 2)],
        ellipsized: false,
    };

    assert!(resolved_text_line_requires_visual_fallback(&line));
    assert!(
        !super::super::projection::line_uses_visual_artifact_projection(
            UiTextWritingMode::HorizontalTb,
            &line,
        ),
        "a physical RTL visual line must not be reshaped as local LTR text"
    );
}

#[test]
fn visual_projection_binds_a_virtual_glyph_to_its_source_anchor() {
    let line = UiResolvedTextLine {
        text: "a…b".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 24.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 5 },
        measured_width: 24.0,
        glyph_advances: vec![4.0, 16.0, 4.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "a".to_string(),
                source_range: UiTextRange { start: 0, end: 1 },
                visual_range: UiTextRange { start: 0, end: 1 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "…".to_string(),
                source_range: UiTextRange { start: 1, end: 1 },
                visual_range: UiTextRange { start: 1, end: 4 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "b".to_string(),
                source_range: UiTextRange { start: 1, end: 2 },
                visual_range: UiTextRange { start: 4, end: 5 },
                direction: UiTextDirection::LeftToRight,
            },
        ],
        ellipsized: true,
    };
    let glyphs = vec![glyph(1, 0..1), glyph(2, 1..4), glyph(3, 4..5)];

    let projected =
        super::super::visual_projection::visual_glyphs_for_visual_line("ab", 0, &line, glyphs)
            .expect("a contiguous virtual run has an explicit source anchor");

    assert_eq!(projected[0].source_range, 0..1);
    assert_eq!(projected[0].advance, 4.0);
    assert!(!projected[0].flags.virtual_glyph);
    assert_eq!(projected[1].source_range, 1..1);
    assert_eq!(projected[1].advance, 16.0);
    assert!(projected[1].flags.virtual_glyph);
    assert_eq!(projected[2].source_range, 1..2);
    assert_eq!(projected[2].advance, 4.0);
    assert!(!projected[2].flags.virtual_glyph);
}

#[test]
fn visual_projection_retains_many_distinct_ltr_virtual_anchors() {
    const ANCHOR_COUNT: usize = 64;
    const ELLIPSIS: &str = "…";
    let source = "a".repeat(ANCHOR_COUNT);
    let line = UiResolvedTextLine {
        text: ELLIPSIS.repeat(ANCHOR_COUNT),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, ANCHOR_COUNT as f32 * 4.0, 12.0),
        source_range: UiTextRange {
            start: 0,
            end: ANCHOR_COUNT,
        },
        visual_range: UiTextRange {
            start: 0,
            end: ANCHOR_COUNT * ELLIPSIS.len(),
        },
        measured_width: ANCHOR_COUNT as f32 * 4.0,
        glyph_advances: vec![4.0; ANCHOR_COUNT],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: (0..ANCHOR_COUNT)
            .map(|index| UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: ELLIPSIS.to_string(),
                source_range: UiTextRange {
                    start: index,
                    end: index,
                },
                visual_range: UiTextRange {
                    start: index * ELLIPSIS.len(),
                    end: (index + 1) * ELLIPSIS.len(),
                },
                direction: UiTextDirection::LeftToRight,
            })
            .collect(),
        ellipsized: true,
    };
    let glyphs = (0..ANCHOR_COUNT)
        .map(|index| {
            glyph(
                index as u32,
                index * ELLIPSIS.len()..(index + 1) * ELLIPSIS.len(),
            )
        })
        .collect();

    let projected =
        super::super::visual_projection::visual_glyphs_for_visual_line(&source, 0, &line, glyphs)
            .expect("monotonic virtual anchors must retain their individual source boundaries");

    assert_eq!(projected.len(), ANCHOR_COUNT);
    for (index, glyph) in projected.iter().enumerate() {
        assert_eq!(glyph.source_range, index..index);
        assert_eq!(glyph.advance, 4.0);
        assert!(glyph.flags.virtual_glyph);
    }
}

#[test]
fn glyph_artifact_owner_republishes_an_ellipsized_virtual_glyph_for_a_new_font_generation() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let (_, database) = crate::text::font::shared_font_database_snapshot();
    let line = UiResolvedTextLine {
        text: "a…b".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 24.0, 20.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 5 },
        measured_width: 24.0,
        glyph_advances: vec![4.0, 16.0, 4.0],
        baseline: 16.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "a".to_string(),
                source_range: UiTextRange { start: 0, end: 1 },
                visual_range: UiTextRange { start: 0, end: 1 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "…".to_string(),
                source_range: UiTextRange { start: 1, end: 1 },
                visual_range: UiTextRange { start: 1, end: 4 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "b".to_string(),
                source_range: UiTextRange { start: 1, end: 2 },
                visual_range: UiTextRange { start: 4, end: 5 },
                direction: UiTextDirection::LeftToRight,
            },
        ],
        ellipsized: true,
    };
    let style = UiResolvedStyle {
        font_size: 16.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    };
    let layout = UiResolvedTextLayout {
        writing_mode: UiTextWritingMode::HorizontalTb,
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width: line.measured_width,
        measured_height: line.frame.height,
        source_range: line.source_range,
        lines: vec![line.clone()],
        ..UiResolvedTextLayout::default()
    };
    let mut provider = SharedTextLayoutSession::new();

    let artifact = build_resolved_text_glyph_artifact("ab", &style, &layout, &mut provider)
        .into_result()
        .expect("an ellipsized visual line must shape")
        .expect("an ellipsized visual line must retain an artifact");
    let glyphs = &artifact.lines[0].as_ref().expect("artifact line").glyphs;
    assert!(glyphs.iter().any(|glyph| {
        glyph.flags.virtual_glyph && glyph.source_range == (1..1) && glyph.advance > 0.0
    }));

    let published_generation = crate::text::font::force_publish_shared_font_database(&database);
    let mut next_generation_provider = SharedTextLayoutSession::new();
    let republished =
        build_resolved_text_glyph_artifact("ab", &style, &layout, &mut next_generation_provider)
            .into_result()
            .expect("the text owner must rebuild the complete ellipsized artifact")
            .expect("the text owner must republish the complete ellipsized artifact");

    assert_eq!(republished.font_generation, published_generation);
    assert_ne!(republished.font_generation, artifact.font_generation);
    assert!(
        republished.lines[0]
            .as_ref()
            .expect("republished artifact line")
            .glyphs
            .iter()
            .any(|glyph| {
                glyph.flags.virtual_glyph && glyph.source_range == (1..1) && glyph.advance > 0.0
            })
    );
}

#[test]
fn artifact_projection_rejects_out_of_order_visual_runs() {
    let line = UiResolvedTextLine {
        text: "ab".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 20.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 20.0,
        glyph_advances: vec![10.0, 10.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![visual_run("a", 0, 1, 1, 2), visual_run("b", 1, 2, 0, 1)],
        ellipsized: false,
    };

    assert!(
        resolved_text_line_requires_visual_fallback(&line),
        "artifact projection requires visual runs in contiguous visual order"
    );
}

#[test]
fn artifact_projection_rejects_incomplete_visual_run_coverage() {
    let line = UiResolvedTextLine {
        text: "ab".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 20.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 20.0,
        glyph_advances: vec![10.0, 10.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![visual_run("a", 0, 1, 0, 1)],
        ellipsized: false,
    };

    assert!(
        resolved_text_line_requires_visual_fallback(&line),
        "artifact projection requires visual runs to cover the full visual range"
    );
}

#[test]
fn secure_presentation_projects_ordered_glyphs_across_many_mask_runs() {
    const CLUSTER_COUNT: usize = 64;
    const MASK: &str = "\u{2022}";
    let text = MASK.repeat(CLUSTER_COUNT);
    let line = UiResolvedTextLine {
        text,
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 640.0, 12.0),
        source_range: UiTextRange {
            start: 0,
            end: CLUSTER_COUNT * 2,
        },
        visual_range: UiTextRange {
            start: 0,
            end: CLUSTER_COUNT * MASK.len(),
        },
        measured_width: 640.0,
        glyph_advances: (0..CLUSTER_COUNT).map(|index| index as f32 + 1.0).collect(),
        baseline: 9.0,
        direction: UiTextDirection::RightToLeft,
        runs: (0..CLUSTER_COUNT)
            .map(|index| {
                visual_run(
                    MASK,
                    index * 2,
                    index * 2 + 2,
                    index * MASK.len(),
                    (index + 1) * MASK.len(),
                )
            })
            .collect(),
        ellipsized: false,
    };
    let glyphs = (0..CLUSTER_COUNT)
        .map(|index| glyph(index as u32, index * MASK.len()..(index + 1) * MASK.len()))
        .collect();

    let projected = super::super::visual_projection::presentation_glyphs_for_line(&line, glyphs)
        .expect("ordered secure mask glyphs must retain their explicit source map");

    assert_eq!(projected.len(), CLUSTER_COUNT);
    for (index, glyph) in projected.iter().enumerate() {
        assert_eq!(glyph.source_range, index * 2..index * 2 + 2);
        assert_eq!(glyph.advance, index as f32 + 1.0);
        assert!(glyph.flags.right_to_left);
    }
}

#[test]
fn secure_presentation_rejects_glyphs_that_rewind_in_visual_order() {
    let line = UiResolvedTextLine {
        text: "\u{2022}\u{2022}".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 20.0, 12.0),
        source_range: UiTextRange { start: 0, end: 4 },
        visual_range: UiTextRange { start: 0, end: 6 },
        measured_width: 20.0,
        glyph_advances: vec![10.0, 10.0],
        baseline: 9.0,
        direction: UiTextDirection::RightToLeft,
        runs: vec![
            visual_run("\u{2022}", 0, 2, 0, 3),
            visual_run("\u{2022}", 2, 4, 3, 6),
        ],
        ellipsized: false,
    };
    let glyphs = vec![glyph(0, 3..6), glyph(1, 0..3)];

    assert!(
        super::super::visual_projection::presentation_glyphs_for_line(&line, glyphs).is_none(),
        "a secure projection must reject a backend glyph sequence that violates physical visual order"
    );
}
