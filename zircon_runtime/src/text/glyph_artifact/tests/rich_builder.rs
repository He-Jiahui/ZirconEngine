use super::*;
use crate::core::framework::text::{TextGlyphFlags, TextGlyphRotation};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiResolvedTextRun, UiTextDirection, UiTextRunKind};

#[test]
fn virtual_cluster_inherits_the_source_span_that_owns_the_consumed_hint() {
    let first_style = TextStyle::default();
    let second_style = TextStyle {
        font_weight: 700,
        ..TextStyle::default()
    };
    let sequence = LogicalVirtualLineSequence::new_with_style_owners(
        Arc::from("a-b"),
        TextDirection::LeftToRight,
        vec![
            TextRange { start: 0, end: 1 },
            TextRange { start: 3, end: 3 },
            TextRange { start: 3, end: 4 },
        ],
        vec![None, Some(TextRange { start: 0, end: 3 }), None],
    )
    .expect("virtual marker sequence");
    let spans = [
        ResolvedRichTextSpan {
            start: 0,
            end: 3,
            style: first_style.clone(),
        },
        ResolvedRichTextSpan {
            start: 3,
            end: 4,
            style: second_style.clone(),
        },
    ];

    let resolved = logical_virtual_style_spans(&sequence, &spans)
        .expect("logical clusters resolve to source styles");

    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].range, TextRange { start: 0, end: 2 });
    assert_eq!(resolved[0].style, first_style);
    assert_eq!(resolved[1].range, TextRange { start: 2, end: 3 });
    assert_eq!(resolved[1].style, second_style);
}

#[test]
fn leading_virtual_cluster_uses_its_explicit_following_style_owner() {
    let first_style = TextStyle::default();
    let second_style = TextStyle {
        font_weight: 700,
        ..TextStyle::default()
    };
    let sequence = LogicalVirtualLineSequence::new_with_style_owners(
        Arc::from("\u{2026}b"),
        TextDirection::LeftToRight,
        vec![
            TextRange { start: 3, end: 3 },
            TextRange { start: 3, end: 4 },
        ],
        vec![Some(TextRange { start: 3, end: 4 }), None],
    )
    .expect("leading virtual marker sequence");
    let spans = [
        ResolvedRichTextSpan {
            start: 0,
            end: 3,
            style: first_style,
        },
        ResolvedRichTextSpan {
            start: 3,
            end: 4,
            style: second_style.clone(),
        },
    ];

    let resolved = logical_virtual_style_spans(&sequence, &spans)
        .expect("explicit owner resolves the leading marker style");

    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].range, TextRange { start: 0, end: 4 });
    assert_eq!(resolved[0].style, second_style);
}

#[test]
fn vertical_generated_marker_gate_uses_typed_roles() {
    let ellipsis =
        LogicalVirtualLineSequence::new_with_source_receipts_external_clusters_and_roles(
            Arc::from("a\u{2026}"),
            TextDirection::LeftToRight,
            vec![
                TextRange { start: 0, end: 1 },
                TextRange { start: 1, end: 1 },
            ],
            vec![None, Some(TextRange { start: 0, end: 1 })],
            vec![None, None],
            vec![false, false],
            vec![None, Some(LogicalVirtualFragmentRole::Ellipsis)],
        )
        .expect("vertical ellipsis sequence");
    let hyphen = LogicalVirtualLineSequence::new_with_source_receipts_external_clusters_and_roles(
        Arc::from("a-"),
        TextDirection::LeftToRight,
        vec![
            TextRange { start: 0, end: 1 },
            TextRange { start: 1, end: 1 },
        ],
        vec![None, Some(TextRange { start: 0, end: 1 })],
        vec![None, Some(TextRange { start: 1, end: 3 })],
        vec![false, false],
        vec![None, Some(LogicalVirtualFragmentRole::DiscretionaryHyphen)],
    )
    .expect("vertical discretionary-hyphen sequence");
    let justification =
        LogicalVirtualLineSequence::new_with_source_receipts_external_clusters_and_roles(
            Arc::from("a-"),
            TextDirection::LeftToRight,
            vec![
                TextRange { start: 0, end: 1 },
                TextRange { start: 1, end: 1 },
            ],
            vec![None, Some(TextRange { start: 0, end: 1 })],
            vec![None, None],
            vec![false, false],
            vec![None, Some(LogicalVirtualFragmentRole::Justification)],
        )
        .expect("vertical unsupported-justification sequence");

    assert!(vertical_sequence_uses_supported_generated_markers(
        &ellipsis
    ));
    assert!(vertical_sequence_uses_supported_generated_markers(&hyphen));
    assert!(!vertical_sequence_uses_supported_generated_markers(
        &justification
    ));
}

#[test]
fn external_cluster_splits_rich_shape_spans_without_becoming_text() {
    let style = TextStyle::default();
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
    .expect("external rich virtual sequence");
    let spans = [ResolvedRichTextSpan {
        start: 0,
        end: 6,
        style: style.clone(),
    }];

    let resolved = logical_virtual_style_spans(&sequence, &spans)
        .expect("text spans remain on both sides of the external block");

    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].range, TextRange { start: 0, end: 1 });
    assert_eq!(resolved[1].range, TextRange { start: 4, end: 7 });
    assert_eq!(resolved[0].style, style);
    assert_eq!(resolved[1].style, style);
}

#[test]
fn ligature_is_owned_once_and_the_continuation_run_gets_an_empty_receipt() {
    let line = UiResolvedTextLine {
        text: "fi".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 8.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 8.0,
        glyph_advances: vec![4.0, 4.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "f".to_string(),
                source_range: UiTextRange { start: 0, end: 1 },
                visual_range: UiTextRange { start: 0, end: 1 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "i".to_string(),
                source_range: UiTextRange { start: 1, end: 2 },
                visual_range: UiTextRange { start: 1, end: 2 },
                direction: UiTextDirection::LeftToRight,
            },
        ],
        ellipsized: false,
    };
    let glyph = TextGlyph {
        glyph_id: 7,
        source_range: 0..2,
        visual_range: 0..2,
        advance: 8.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags::default(),
        requires_rasterization: true,
    };

    let runs = glyph_run_ranges("fi", 0, 0, &line, &[glyph], None).expect("run receipts");

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].glyph_range, 0..1);
    assert_eq!(runs[1].glyph_range, 1..1);
}

#[test]
fn virtual_glyph_is_owned_by_the_zero_width_visual_run() {
    let anchor = 5;
    let line = UiResolvedTextLine {
        text: "pre-".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 12.0, 12.0),
        source_range: UiTextRange {
            start: 0,
            end: anchor,
        },
        visual_range: UiTextRange { start: 0, end: 4 },
        measured_width: 12.0,
        glyph_advances: vec![3.0; 4],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "pre".to_string(),
                source_range: UiTextRange { start: 0, end: 3 },
                visual_range: UiTextRange { start: 0, end: 3 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "-".to_string(),
                source_range: UiTextRange {
                    start: anchor,
                    end: anchor,
                },
                visual_range: UiTextRange { start: 3, end: 4 },
                direction: UiTextDirection::LeftToRight,
            },
        ],
        ellipsized: false,
    };
    let glyph = TextGlyph {
        glyph_id: 9,
        source_range: anchor..anchor,
        visual_range: 3..4,
        advance: 3.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags {
            virtual_glyph: true,
            ..TextGlyphFlags::default()
        },
        requires_rasterization: true,
    };

    let sequence = LogicalVirtualLineSequence::new_with_source_receipts(
        Arc::from("pre-"),
        TextDirection::LeftToRight,
        vec![
            TextRange { start: 0, end: 1 },
            TextRange { start: 1, end: 2 },
            TextRange { start: 2, end: 3 },
            TextRange {
                start: anchor,
                end: anchor,
            },
        ],
        vec![
            None,
            None,
            None,
            Some(TextRange {
                start: anchor - 2,
                end: anchor,
            }),
        ],
        vec![
            None,
            None,
            None,
            Some(TextRange {
                start: anchor - 2,
                end: anchor,
            }),
        ],
    )
    .expect("virtual soft hyphen sequence");
    let runs = glyph_run_ranges("pre\u{00ad}", 0, 0, &line, &[glyph], Some(&sequence))
        .expect("virtual glyph run receipt");

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].glyph_range, 0..0);
    assert_eq!(runs[1].glyph_range, 0..1);
    assert_eq!(
        runs[1].style_source_range,
        Some(UiTextRange {
            start: anchor - 2,
            end: anchor,
        })
    );
    assert_eq!(
        runs[1].replaced_source_range,
        Some(UiTextRange {
            start: anchor - 2,
            end: anchor,
        })
    );
}
