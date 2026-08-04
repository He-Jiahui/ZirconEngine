use super::*;
use crate::core::framework::text::{
    TextGlyphFlags, TextGlyphRotation, TextShapeResult, TextShapeRun,
};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiTextCaret, UiTextCaretAffinity, UiTextDirection, UiTextRunKind, UiTextWritingMode,
};

#[test]
fn virtual_source_run_requires_visual_fallback_instead_of_artifact_reshaping() {
    let line = UiResolvedTextLine {
        text: "ـ".to_string(),
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
}

#[test]
fn visual_glyph_artifact_keeps_contextual_arabic_glyphs_in_visual_order() {
    let line = UiResolvedTextLine {
        text: "مالس".to_string(),
        frame: UiFrame::new(0.0, 0.0, 40.0, 12.0),
        source_range: UiTextRange { start: 0, end: 8 },
        visual_range: UiTextRange { start: 0, end: 8 },
        measured_width: 40.0,
        glyph_advances: vec![10.0; 4],
        baseline: 9.0,
        direction: UiTextDirection::RightToLeft,
        runs: vec![
            visual_run("م", 6, 8, 0, 2),
            visual_run("ا", 4, 6, 2, 4),
            visual_run("ل", 2, 4, 4, 6),
            visual_run("س", 0, 2, 6, 8),
        ],
        ellipsized: false,
    };

    let glyphs = visual_glyphs_for_line(
        "سلام",
        0,
        &line,
        TextShapeResult {
            runs: vec![TextShapeRun {
                source_range: 0..8,
                direction: crate::core::framework::text::TextDirection::RightToLeft,
                glyphs: vec![
                    glyph(101, 0..2),
                    glyph(102, 2..4),
                    glyph(103, 4..6),
                    glyph(104, 6..8),
                ],
            }],
            metrics: Default::default(),
            resolved_direction: crate::core::framework::text::TextDirection::RightToLeft,
        },
    );

    assert_eq!(
        glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![104, 103, 102, 101]
    );
}

#[test]
fn visual_glyph_artifact_projects_resolved_advance_to_an_unsplit_ligature() {
    let line = UiResolvedTextLine {
        text: "fi".to_string(),
        frame: UiFrame::new(0.0, 0.0, 30.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 30.0,
        glyph_advances: vec![12.0, 18.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![zircon_runtime_interface::ui::surface::UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "fi".to_string(),
            source_range: UiTextRange { start: 0, end: 2 },
            visual_range: UiTextRange { start: 0, end: 2 },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };

    let glyphs = visual_glyphs_for_line(
        "fi",
        0,
        &line,
        TextShapeResult {
            runs: vec![TextShapeRun {
                source_range: 0..2,
                direction: crate::core::framework::text::TextDirection::LeftToRight,
                glyphs: vec![glyph(77, 0..2)],
            }],
            metrics: Default::default(),
            resolved_direction: crate::core::framework::text::TextDirection::LeftToRight,
        },
    );

    assert_eq!(glyphs.len(), 1);
    assert_eq!(glyphs[0].advance, 30.0);
}

#[test]
fn artifact_cluster_geometry_snaps_ligature_caret_and_selection_to_whole_glyph() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = UiResolvedTextLine {
        text: "fi".to_string(),
        frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 30.0,
        glyph_advances: vec![12.0, 18.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![zircon_runtime_interface::ui::surface::UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "fi".to_string(),
            source_range: UiTextRange { start: 0, end: 2 },
            visual_range: UiTextRange { start: 0, end: 2 },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };
    let mut ligature = glyph(77, 0..2);
    ligature.advance = 30.0;
    ligature.flags.right_to_left = false;
    let artifact = ResolvedTextGlyphArtifact {
        source_text: Arc::from("fi"),
        source_text_origin: 0,
        font_generation: shared_font_database_generation(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![ligature],
            layout_line: line.clone(),
        })],
    };

    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 1,
                affinity: UiTextCaretAffinity::Upstream,
            },
        ),
        Some(0.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 1,
                affinity: UiTextCaretAffinity::Downstream,
            },
        ),
        Some(30.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_range_advance_spans(
            &artifact,
            0,
            &line,
            UiTextRange { start: 0, end: 1 },
        ),
        Some(vec![(0.0, 30.0)])
    );
    let mut mismatched_line = line.clone();
    mismatched_line.glyph_advances[0] = 13.0;
    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &mismatched_line,
            &UiTextCaret {
                offset: 1,
                affinity: UiTextCaretAffinity::Downstream,
            },
        ),
        None
    );
}

#[test]
fn artifact_cluster_geometry_maps_rtl_affinity_to_opposite_visual_edges() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = UiResolvedTextLine {
        text: "fi".to_string(),
        frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 30.0,
        glyph_advances: vec![12.0, 18.0],
        baseline: 9.0,
        direction: UiTextDirection::RightToLeft,
        runs: vec![zircon_runtime_interface::ui::surface::UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "fi".to_string(),
            source_range: UiTextRange { start: 0, end: 2 },
            visual_range: UiTextRange { start: 0, end: 2 },
            direction: UiTextDirection::RightToLeft,
        }],
        ellipsized: false,
    };
    let mut ligature = glyph(77, 0..2);
    ligature.advance = 30.0;
    let artifact = ResolvedTextGlyphArtifact {
        source_text: Arc::from("fi"),
        source_text_origin: 0,
        font_generation: shared_font_database_generation(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![ligature],
            layout_line: line.clone(),
        })],
    };

    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 1,
                affinity: UiTextCaretAffinity::Upstream,
            },
        ),
        Some(30.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 1,
                affinity: UiTextCaretAffinity::Downstream,
            },
        ),
        Some(0.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_at_advance(&artifact, 0, &line, 12.0),
        Some(UiTextCaret {
            offset: 2,
            affinity: UiTextCaretAffinity::Downstream,
        })
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_at_advance(&artifact, 0, &line, 18.0),
        Some(UiTextCaret {
            offset: 0,
            affinity: UiTextCaretAffinity::Upstream,
        })
    );
    assert_eq!(
        resolved_text_glyph_artifact_range_advance_spans(
            &artifact,
            0,
            &line,
            UiTextRange { start: 0, end: 1 },
        ),
        Some(vec![(0.0, 30.0)])
    );
}

#[test]
fn artifact_cluster_geometry_merges_multiglyph_backend_clusters() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = UiResolvedTextLine {
        text: "fi".to_string(),
        frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 30.0,
        glyph_advances: vec![12.0, 18.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![visual_run("fi", 0, 2, 0, 2)],
        ellipsized: false,
    };
    let mut leading = glyph(77, 0..1);
    leading.advance = 10.0;
    leading.flags.right_to_left = false;
    leading.flags.cluster_start = true;
    let trailing = TextGlyph {
        glyph_id: 78,
        source_range: 1..2,
        visual_range: 1..2,
        advance: 20.0,
        flags: TextGlyphFlags::default(),
        ..leading.clone()
    };
    let artifact = ResolvedTextGlyphArtifact {
        source_text: Arc::from("fi"),
        source_text_origin: 0,
        font_generation: shared_font_database_generation(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![leading, trailing],
            layout_line: line.clone(),
        })],
    };

    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 1,
                affinity: UiTextCaretAffinity::Upstream,
            },
        ),
        Some(0.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 1,
                affinity: UiTextCaretAffinity::Downstream,
            },
        ),
        Some(30.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_at_advance(&artifact, 0, &line, 12.0),
        Some(UiTextCaret {
            offset: 0,
            affinity: UiTextCaretAffinity::Downstream,
        })
    );
    assert_eq!(
        resolved_text_glyph_artifact_range_advance_spans(
            &artifact,
            0,
            &line,
            UiTextRange { start: 0, end: 1 },
        ),
        Some(vec![(0.0, 30.0)])
    );
}

#[test]
fn artifact_cluster_geometry_rejects_a_stale_font_generation() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = UiResolvedTextLine {
        text: "fi".to_string(),
        frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 30.0,
        glyph_advances: vec![12.0, 18.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![zircon_runtime_interface::ui::surface::UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "fi".to_string(),
            source_range: UiTextRange { start: 0, end: 2 },
            visual_range: UiTextRange { start: 0, end: 2 },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };
    let mut ligature = glyph(77, 0..2);
    ligature.advance = 30.0;
    ligature.flags.right_to_left = false;
    let artifact = ResolvedTextGlyphArtifact {
        source_text: Arc::from("fi"),
        source_text_origin: 0,
        font_generation: shared_font_database_generation().wrapping_add(1),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![ligature],
            layout_line: line.clone(),
        })],
    };
    let caret = UiTextCaret {
        offset: 1,
        affinity: UiTextCaretAffinity::Downstream,
    };

    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(&artifact, 0, &line, &caret),
        None
    );
    assert_eq!(
        resolved_text_glyph_artifact_range_advance_spans(
            &artifact,
            0,
            &line,
            UiTextRange { start: 0, end: 1 },
        ),
        None
    );
}

#[test]
fn visual_glyph_artifact_preserves_tab_and_justified_space_advances() {
    let line = UiResolvedTextLine {
        text: "a\tb c".to_string(),
        frame: UiFrame::new(0.0, 0.0, 91.0, 12.0),
        source_range: UiTextRange { start: 0, end: 5 },
        visual_range: UiTextRange { start: 0, end: 5 },
        measured_width: 91.0,
        glyph_advances: vec![9.0, 40.0, 9.0, 24.0, 9.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![visual_run("a\tb c", 0, 5, 0, 5)],
        ellipsized: false,
    };

    let glyphs = visual_glyphs_for_line(
        "a\tb c",
        0,
        &line,
        TextShapeResult {
            runs: vec![TextShapeRun {
                source_range: 0..5,
                direction: crate::core::framework::text::TextDirection::LeftToRight,
                glyphs: vec![
                    glyph(1, 0..1),
                    glyph(2, 1..2),
                    glyph(3, 2..3),
                    glyph(4, 3..4),
                    glyph(5, 4..5),
                ],
            }],
            metrics: Default::default(),
            resolved_direction: crate::core::framework::text::TextDirection::LeftToRight,
        },
    );

    assert_eq!(
        glyphs.iter().map(|glyph| glyph.advance).collect::<Vec<_>>(),
        line.glyph_advances
    );
}

fn visual_run(
    text: &str,
    source_start: usize,
    source_end: usize,
    visual_start: usize,
    visual_end: usize,
) -> zircon_runtime_interface::ui::surface::UiResolvedTextRun {
    UiResolvedTextRun {
        kind: UiTextRunKind::Plain,
        text: text.to_string(),
        source_range: UiTextRange {
            start: source_start,
            end: source_end,
        },
        visual_range: UiTextRange {
            start: visual_start,
            end: visual_end,
        },
        direction: UiTextDirection::RightToLeft,
    }
}

fn glyph(glyph_id: u32, source_range: std::ops::Range<usize>) -> TextGlyph {
    TextGlyph {
        glyph_id,
        source_range,
        visual_range: 0..0,
        advance: 10.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 1,
        flags: TextGlyphFlags {
            right_to_left: true,
            ..TextGlyphFlags::default()
        },
        requires_rasterization: true,
    }
}
