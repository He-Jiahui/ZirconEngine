use super::*;

#[test]
fn artifact_cluster_geometry_snaps_ligature_caret_and_selection_to_whole_glyph() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = UiResolvedTextLine {
        text: "fi".to_string(),
        placement_frame: UiFrame::default(),
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
        font_lease: ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![ligature],
            layout_line: line.clone(),
        })],
        logical_virtual_line_sequences: None,
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
        placement_frame: UiFrame::default(),
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
        font_lease: ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![ligature],
            layout_line: line.clone(),
        })],
        logical_virtual_line_sequences: None,
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
        placement_frame: UiFrame::default(),
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
        font_lease: ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![leading, trailing],
            layout_line: line.clone(),
        })],
        logical_virtual_line_sequences: None,
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
        placement_frame: UiFrame::default(),
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
        font_lease: ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![ligature],
            layout_line: line.clone(),
        })],
        logical_virtual_line_sequences: None,
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
