use super::*;

#[test]
fn omitted_source_maps_to_the_ellipsis_marker_geometry() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = UiResolvedTextLine {
        text: "a\u{2026}f".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(10.0, 20.0, 28.0, 12.0),
        source_range: UiTextRange { start: 0, end: 6 },
        visual_range: UiTextRange { start: 0, end: 5 },
        measured_width: 28.0,
        glyph_advances: vec![10.0, 8.0, 10.0],
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
                text: "\u{2026}".to_string(),
                source_range: UiTextRange { start: 6, end: 6 },
                visual_range: UiTextRange { start: 1, end: 4 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "f".to_string(),
                source_range: UiTextRange { start: 5, end: 6 },
                visual_range: UiTextRange { start: 4, end: 5 },
                direction: UiTextDirection::LeftToRight,
            },
        ],
        ellipsized: true,
    };
    let mut leading = glyph(1, 0..1);
    leading.advance = 10.0;
    leading.bidi_level = 0;
    leading.flags = TextGlyphFlags {
        cluster_start: true,
        ..TextGlyphFlags::default()
    };
    let mut marker = glyph(2, 6..6);
    marker.advance = 8.0;
    marker.bidi_level = 0;
    marker.flags = TextGlyphFlags {
        cluster_start: true,
        virtual_glyph: true,
        ..TextGlyphFlags::default()
    };
    let mut trailing = glyph(3, 5..6);
    trailing.advance = 10.0;
    trailing.bidi_level = 0;
    trailing.flags = TextGlyphFlags {
        cluster_start: true,
        ..TextGlyphFlags::default()
    };
    let sequence = LogicalVirtualLineSequence::new_with_source_receipts(
        Arc::from("a\u{2026}f"),
        TextDirection::LeftToRight,
        vec![
            TextRange { start: 0, end: 1 },
            TextRange { start: 6, end: 6 },
            TextRange { start: 5, end: 6 },
        ],
        vec![None, Some(TextRange { start: 0, end: 1 }), None],
        vec![None, Some(TextRange { start: 1, end: 5 }), None],
    )
    .expect("ellipsis sequence");
    let artifact = ResolvedTextGlyphArtifact {
        source_text: Arc::from("abcdef"),
        source_text_origin: 0,
        font_generation: shared_font_database_generation(),
        font_lease: ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![leading, marker, trailing],
            layout_line: line.clone(),
        })],
        logical_virtual_line_sequences: Some(vec![Some(sequence)]),
    };

    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 3,
                affinity: UiTextCaretAffinity::Upstream,
            },
        ),
        Some(10.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 3,
                affinity: UiTextCaretAffinity::Downstream,
            },
        ),
        Some(18.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_at_advance(&artifact, 0, &line, 12.0),
        Some(UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        })
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_at_advance(&artifact, 0, &line, 16.0),
        Some(UiTextCaret {
            offset: 5,
            affinity: UiTextCaretAffinity::Upstream,
        })
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_at_advance(&artifact, 0, &line, 18.0),
        Some(UiTextCaret {
            offset: 5,
            affinity: UiTextCaretAffinity::Downstream,
        })
    );
    assert_eq!(
        resolved_text_glyph_artifact_range_advance_spans(
            &artifact,
            0,
            &line,
            UiTextRange { start: 2, end: 4 },
        ),
        Some(vec![(10.0, 18.0)])
    );
    assert_eq!(
        resolved_text_glyph_artifact_range_advance_spans(
            &artifact,
            0,
            &line,
            UiTextRange { start: 0, end: 6 },
        ),
        Some(vec![(0.0, 28.0)])
    );
}

#[test]
fn external_cluster_and_ellipsis_share_one_visual_geometry_owner() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = UiResolvedTextLine {
        text: "a\u{fffc}\u{2026}".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 34.0, 12.0),
        source_range: UiTextRange { start: 0, end: 6 },
        visual_range: UiTextRange { start: 0, end: 7 },
        measured_width: 34.0,
        glyph_advances: vec![10.0, 16.0, 8.0],
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
                text: "\u{fffc}".to_string(),
                source_range: UiTextRange { start: 1, end: 4 },
                visual_range: UiTextRange { start: 1, end: 4 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "\u{2026}".to_string(),
                source_range: UiTextRange { start: 6, end: 6 },
                visual_range: UiTextRange { start: 4, end: 7 },
                direction: UiTextDirection::LeftToRight,
            },
        ],
        ellipsized: true,
    };
    let mut leading = glyph(1, 0..1);
    leading.advance = 10.0;
    leading.flags.cluster_start = true;
    let mut marker = glyph(2, 6..6);
    marker.advance = 8.0;
    marker.flags = TextGlyphFlags {
        cluster_start: true,
        virtual_glyph: true,
        ..TextGlyphFlags::default()
    };
    let sequence = LogicalVirtualLineSequence::new_with_source_receipts_and_external_clusters(
        Arc::from("a\u{fffc}\u{2026}"),
        TextDirection::LeftToRight,
        vec![
            TextRange { start: 0, end: 1 },
            TextRange { start: 1, end: 4 },
            TextRange { start: 6, end: 6 },
        ],
        vec![None, None, Some(TextRange { start: 4, end: 6 })],
        vec![None, None, Some(TextRange { start: 4, end: 6 })],
        vec![false, true, false],
    )
    .expect("external ellipsis sequence");
    let artifact = ResolvedTextGlyphArtifact {
        source_text: Arc::from("a\u{fffc}bc"),
        source_text_origin: 0,
        font_generation: shared_font_database_generation(),
        font_lease: ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![leading, marker],
            layout_line: line.clone(),
        })],
        logical_virtual_line_sequences: Some(vec![Some(sequence)]),
    };

    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 2,
                affinity: UiTextCaretAffinity::Upstream,
            },
        ),
        Some(10.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_advance(
            &artifact,
            0,
            &line,
            &UiTextCaret {
                offset: 2,
                affinity: UiTextCaretAffinity::Downstream,
            },
        ),
        Some(26.0)
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_at_advance(&artifact, 0, &line, 18.0),
        Some(UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        })
    );
    assert_eq!(
        resolved_text_glyph_artifact_caret_at_advance(&artifact, 0, &line, 24.0),
        Some(UiTextCaret {
            offset: 4,
            affinity: UiTextCaretAffinity::Upstream,
        })
    );
    assert_eq!(
        resolved_text_glyph_artifact_range_advance_spans(
            &artifact,
            0,
            &line,
            UiTextRange { start: 1, end: 4 },
        ),
        Some(vec![(10.0, 26.0)])
    );
    assert_eq!(
        resolved_text_glyph_artifact_range_advance_spans(
            &artifact,
            0,
            &line,
            UiTextRange { start: 0, end: 6 },
        ),
        Some(vec![(0.0, 34.0)])
    );
}
