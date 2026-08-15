use super::source_map::*;
use super::*;
use crate::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextRun, UiTextByteRange, UiTextComposition, UiTextDirection,
        UiTextPaintDecorationKind, UiTextPreeditClause, UiTextPreeditClauseKind, UiTextRunKind,
        UiTextSelection,
    },
};

fn mixed_bidi_line() -> UiResolvedTextLine {
    UiResolvedTextLine {
        text: "abc בא".to_string(),
        frame: UiFrame::new(10.0, 20.0, 60.0, 12.0),
        source_range: UiTextRange { start: 0, end: 8 },
        visual_range: UiTextRange { start: 0, end: 8 },
        measured_width: 60.0,
        glyph_advances: vec![10.0; 6],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![
            run("abc ", 0, 4, 0, 4, UiTextDirection::LeftToRight),
            run("ב", 6, 8, 4, 6, UiTextDirection::RightToLeft),
            run("א", 4, 6, 6, 8, UiTextDirection::RightToLeft),
        ],
        ellipsized: false,
    }
}

#[test]
fn mixed_bidi_logical_boundary_affinity_selects_distinct_visual_edges() {
    let line = mixed_bidi_line();
    let map = UiTextLineSourceMap::new(&line);

    let upstream = map.visual_offset_for_caret(&UiTextCaret {
        offset: 4,
        affinity: UiTextCaretAffinity::Upstream,
    });
    let downstream = map.visual_offset_for_caret(&UiTextCaret {
        offset: 4,
        affinity: UiTextCaretAffinity::Downstream,
    });

    assert_eq!(upstream, 4);
    assert_eq!(downstream, 8);
    assert_eq!(map.advance_to_visual_offset(upstream), 40.0);
    assert_eq!(map.advance_to_visual_offset(downstream), 60.0);
}

#[test]
fn visual_advance_prefix_preserves_non_uniform_grapheme_boundaries() {
    let line = UiResolvedTextLine {
        text: "a\u{754c}e\u{301}".to_string(),
        frame: UiFrame::new(10.0, 20.0, 21.0, 12.0),
        source_range: UiTextRange { start: 0, end: 7 },
        visual_range: UiTextRange { start: 0, end: 7 },
        measured_width: 21.0,
        glyph_advances: vec![3.0, 7.0, 11.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![run(
            "a\u{754c}e\u{301}",
            0,
            7,
            0,
            7,
            UiTextDirection::LeftToRight,
        )],
        ellipsized: false,
    };
    let map = UiTextLineSourceMap::new(&line);

    assert_eq!(map.advance_to_visual_offset(0), 0.0);
    assert_eq!(map.advance_to_visual_offset(1), 3.0);
    assert_eq!(map.advance_to_visual_offset(4), 10.0);
    assert_eq!(map.advance_to_visual_offset(7), 21.0);
}

#[test]
fn text_ime_preedit_span_injected_with_real_metrics() {
    let layout = UiResolvedTextLayout {
        lines: vec![UiResolvedTextLine {
            text: "ab".to_string(),
            frame: UiFrame::new(10.0, 20.0, 20.0, 12.0),
            source_range: UiTextRange { start: 0, end: 2 },
            visual_range: UiTextRange { start: 0, end: 2 },
            measured_width: 20.0,
            glyph_advances: vec![10.0, 10.0],
            baseline: 9.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![run("ab", 0, 2, 0, 2, UiTextDirection::LeftToRight)],
            ellipsized: false,
        }],
        ..Default::default()
    };
    let editable = UiEditableTextState {
        text: "ab".to_string(),
        caret: UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        },
        composition: Some(UiTextComposition {
            range: UiTextRange { start: 0, end: 2 },
            text: "ab".to_string(),
            preedit_clauses: vec![
                UiTextPreeditClause::new(
                    UiTextByteRange::new(0, 1),
                    UiTextPreeditClauseKind::Input,
                ),
                UiTextPreeditClause::new(
                    UiTextByteRange::new(1, 2),
                    UiTextPreeditClauseKind::TargetNotConverted,
                ),
            ],
            restore_text: None,
        }),
        ..Default::default()
    };

    let decorations = editable_text_decorations(&layout, &editable);
    let caret = decorations
        .iter()
        .find(|decoration| decoration.kind == UiTextPaintDecorationKind::Caret)
        .expect("preedit span owns an in-span caret");
    let highlights = decorations
        .iter()
        .filter(|decoration| {
            matches!(
                decoration.kind,
                UiTextPaintDecorationKind::CompositionHighlight
            )
        })
        .map(|decoration| {
            (
                decoration.range,
                decoration.frame,
                decoration.color.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let underlines = decorations
        .iter()
        .filter(|decoration| {
            matches!(
                decoration.kind,
                UiTextPaintDecorationKind::CompositionUnderline
            )
        })
        .map(|decoration| (decoration.range, decoration.color.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        highlights,
        vec![(
            UiTextRange { start: 0, end: 2 },
            UiFrame::new(10.0, 20.0, 20.0, 12.0),
            "#4d89ff24",
        )]
    );
    assert_eq!(
        underlines,
        vec![
            (UiTextRange { start: 0, end: 1 }, "#4d89ff"),
            (UiTextRange { start: 1, end: 2 }, "#e05a5a"),
        ]
    );
    assert_eq!(caret.frame, UiFrame::new(20.0, 20.0, 1.0, 12.0));
}

#[test]
fn multiline_selection_and_preedit_clauses_preserve_paint_order() {
    let layout = UiResolvedTextLayout {
        lines: vec![
            UiResolvedTextLine {
                text: "ab".to_string(),
                frame: UiFrame::new(10.0, 20.0, 20.0, 12.0),
                source_range: UiTextRange { start: 0, end: 2 },
                visual_range: UiTextRange { start: 0, end: 2 },
                measured_width: 20.0,
                glyph_advances: vec![10.0, 10.0],
                baseline: 9.0,
                direction: UiTextDirection::LeftToRight,
                runs: vec![run("ab", 0, 2, 0, 2, UiTextDirection::LeftToRight)],
                ellipsized: false,
            },
            UiResolvedTextLine {
                text: "cd".to_string(),
                frame: UiFrame::new(10.0, 32.0, 20.0, 12.0),
                source_range: UiTextRange { start: 2, end: 4 },
                visual_range: UiTextRange { start: 0, end: 2 },
                measured_width: 20.0,
                glyph_advances: vec![10.0, 10.0],
                baseline: 9.0,
                direction: UiTextDirection::LeftToRight,
                runs: vec![run("cd", 2, 4, 0, 2, UiTextDirection::LeftToRight)],
                ellipsized: false,
            },
        ],
        ..Default::default()
    };
    let editable = UiEditableTextState {
        text: "abcd".to_string(),
        caret: UiTextCaret {
            offset: 4,
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection: Some(UiTextSelection {
            anchor: 0,
            focus: 4,
        }),
        composition: Some(UiTextComposition {
            range: UiTextRange { start: 0, end: 4 },
            text: "abcd".to_string(),
            preedit_clauses: vec![
                UiTextPreeditClause::new(
                    UiTextByteRange::new(0, 1),
                    UiTextPreeditClauseKind::Input,
                ),
                UiTextPreeditClause::new(
                    UiTextByteRange::new(1, 3),
                    UiTextPreeditClauseKind::Converted,
                ),
                UiTextPreeditClause::new(
                    UiTextByteRange::new(3, 4),
                    UiTextPreeditClauseKind::TargetConverted,
                ),
            ],
            restore_text: None,
        }),
        ..Default::default()
    };

    let decorations = editable_text_decorations(&layout, &editable)
        .into_iter()
        .filter(|decoration| {
            matches!(
                decoration.kind,
                UiTextPaintDecorationKind::Selection
                    | UiTextPaintDecorationKind::CompositionHighlight
                    | UiTextPaintDecorationKind::CompositionUnderline
            )
        })
        .map(|decoration| {
            (
                decoration.kind,
                decoration.range,
                decoration.color,
                decoration.frame,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        decorations,
        vec![
            (
                UiTextPaintDecorationKind::CompositionHighlight,
                UiTextRange { start: 0, end: 4 },
                "#4d89ff24".to_string(),
                UiFrame::new(10.0, 20.0, 20.0, 12.0),
            ),
            (
                UiTextPaintDecorationKind::CompositionHighlight,
                UiTextRange { start: 0, end: 4 },
                "#4d89ff24".to_string(),
                UiFrame::new(10.0, 32.0, 20.0, 12.0),
            ),
            (
                UiTextPaintDecorationKind::Selection,
                UiTextRange { start: 0, end: 4 },
                "#4d89ff66".to_string(),
                UiFrame::new(10.0, 20.0, 20.0, 12.0),
            ),
            (
                UiTextPaintDecorationKind::Selection,
                UiTextRange { start: 0, end: 4 },
                "#4d89ff66".to_string(),
                UiFrame::new(10.0, 32.0, 20.0, 12.0),
            ),
            (
                UiTextPaintDecorationKind::CompositionUnderline,
                UiTextRange { start: 0, end: 1 },
                "#4d89ff".to_string(),
                UiFrame::new(10.0, 30.0, 10.0, 2.0),
            ),
            (
                UiTextPaintDecorationKind::CompositionUnderline,
                UiTextRange { start: 1, end: 3 },
                "#72b7f2".to_string(),
                UiFrame::new(20.0, 30.0, 10.0, 2.0),
            ),
            (
                UiTextPaintDecorationKind::CompositionUnderline,
                UiTextRange { start: 1, end: 3 },
                "#72b7f2".to_string(),
                UiFrame::new(10.0, 42.0, 10.0, 2.0),
            ),
            (
                UiTextPaintDecorationKind::CompositionUnderline,
                UiTextRange { start: 3, end: 4 },
                "#42bf77".to_string(),
                UiFrame::new(20.0, 42.0, 10.0, 2.0),
            ),
        ]
    );
}

#[test]
fn preedit_clauses_preserve_multibyte_utf8_source_ranges() {
    let layout = UiResolvedTextLayout {
        lines: vec![UiResolvedTextLine {
            text: "a\u{754c}b".to_string(),
            frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
            source_range: UiTextRange { start: 0, end: 5 },
            visual_range: UiTextRange { start: 0, end: 5 },
            measured_width: 30.0,
            glyph_advances: vec![10.0, 10.0, 10.0],
            baseline: 9.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![run("a\u{754c}b", 0, 5, 0, 5, UiTextDirection::LeftToRight)],
            ellipsized: false,
        }],
        ..Default::default()
    };
    let editable = UiEditableTextState {
        text: "a\u{754c}b".to_string(),
        caret: UiTextCaret {
            offset: 5,
            affinity: UiTextCaretAffinity::Downstream,
        },
        composition: Some(UiTextComposition {
            range: UiTextRange { start: 0, end: 5 },
            text: "a\u{754c}b".to_string(),
            preedit_clauses: vec![
                UiTextPreeditClause::new(
                    UiTextByteRange::new(1, 4),
                    UiTextPreeditClauseKind::Converted,
                ),
                UiTextPreeditClause::new(
                    UiTextByteRange::new(4, 5),
                    UiTextPreeditClauseKind::TargetConverted,
                ),
            ],
            restore_text: None,
        }),
        ..Default::default()
    };

    let underlines = editable_text_decorations(&layout, &editable)
        .into_iter()
        .filter(|decoration| {
            matches!(
                decoration.kind,
                UiTextPaintDecorationKind::CompositionUnderline
            )
        })
        .map(|decoration| (decoration.range, decoration.color))
        .collect::<Vec<_>>();

    assert_eq!(
        underlines,
        vec![
            (UiTextRange { start: 1, end: 4 }, "#72b7f2".to_string()),
            (UiTextRange { start: 4, end: 5 }, "#42bf77".to_string()),
        ]
    );
}

#[test]
fn mismatched_glyph_advances_keep_legacy_proportional_fallback() {
    let line = UiResolvedTextLine {
        text: "ab\u{754c}".to_string(),
        frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
        source_range: UiTextRange { start: 0, end: 5 },
        visual_range: UiTextRange { start: 0, end: 5 },
        measured_width: 30.0,
        glyph_advances: vec![30.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![run("ab\u{754c}", 0, 5, 0, 5, UiTextDirection::LeftToRight)],
        ellipsized: false,
    };
    let map = UiTextLineSourceMap::new(&line);

    assert_eq!(map.advance_to_visual_offset(0), 0.0);
    assert_eq!(map.advance_to_visual_offset(1), 10.0);
    assert_eq!(map.advance_to_visual_offset(2), 20.0);
    assert_eq!(map.advance_to_visual_offset(5), 30.0);
}

#[test]
fn isomorphic_single_run_preserves_internal_ltr_caret_and_selection_offsets() {
    let line = mixed_bidi_line();
    let map = UiTextLineSourceMap::new(&line);

    for affinity in [
        UiTextCaretAffinity::Upstream,
        UiTextCaretAffinity::Downstream,
    ] {
        assert_eq!(
            map.visual_offset_for_caret(&UiTextCaret {
                offset: 1,
                affinity,
            }),
            1,
            "the internal LTR caret must retain its source-relative visual boundary"
        );
    }
    assert_eq!(
        map.caret_for_visual_boundary(
            1,
            UiTextVisualBoundaryBias::LeadingCurrent,
            line.source_range.start,
        ),
        UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Downstream,
        }
    );
    assert_eq!(
        map.caret_for_visual_boundary(
            1,
            UiTextVisualBoundaryBias::TrailingPrevious,
            line.source_range.end,
        ),
        UiTextCaret {
            offset: 1,
            affinity: UiTextCaretAffinity::Upstream,
        }
    );
    assert_eq!(
        map.visual_spans_for_source_range(UiTextRange { start: 1, end: 2 }),
        vec![UiTextVisualSpan {
            visual_range: UiTextRange { start: 1, end: 2 },
        }]
    );
}

#[test]
fn isomorphic_single_rtl_run_reverses_internal_source_offsets() {
    let line = UiResolvedTextLine {
        text: "\u{5d1}\u{5d0}".to_string(),
        frame: UiFrame::new(10.0, 20.0, 20.0, 12.0),
        source_range: UiTextRange { start: 0, end: 4 },
        visual_range: UiTextRange { start: 0, end: 4 },
        measured_width: 20.0,
        glyph_advances: vec![10.0, 10.0],
        baseline: 9.0,
        direction: UiTextDirection::RightToLeft,
        runs: vec![run(
            "\u{5d1}\u{5d0}",
            0,
            4,
            0,
            4,
            UiTextDirection::RightToLeft,
        )],
        ellipsized: false,
    };
    let map = UiTextLineSourceMap::new(&line);

    assert_eq!(
        map.caret_for_visual_boundary(
            1,
            UiTextVisualBoundaryBias::LeadingCurrent,
            line.source_range.start,
        ),
        UiTextCaret {
            offset: 2,
            affinity: UiTextCaretAffinity::Downstream,
        }
    );
    for affinity in [
        UiTextCaretAffinity::Upstream,
        UiTextCaretAffinity::Downstream,
    ] {
        assert_eq!(
            map.visual_offset_for_caret(&UiTextCaret {
                offset: 2,
                affinity,
            }),
            2
        );
    }
}

#[test]
fn mixed_bidi_source_range_projects_discontiguous_visual_spans() {
    let line = mixed_bidi_line();
    let map = UiTextLineSourceMap::new(&line);

    assert_eq!(
        map.visual_spans_for_source_range(UiTextRange { start: 0, end: 6 }),
        vec![
            UiTextVisualSpan {
                visual_range: UiTextRange { start: 0, end: 4 },
            },
            UiTextVisualSpan {
                visual_range: UiTextRange { start: 6, end: 8 },
            },
        ]
    );
}

#[test]
fn visual_rtl_edges_round_trip_to_logical_offsets_and_affinity() {
    let line = mixed_bidi_line();
    let map = UiTextLineSourceMap::new(&line);

    assert_eq!(
        map.caret_for_visual_boundary(
            4,
            UiTextVisualBoundaryBias::LeadingCurrent,
            line.source_range.start,
        ),
        UiTextCaret {
            offset: 8,
            affinity: UiTextCaretAffinity::Downstream,
        }
    );
    assert_eq!(
        map.caret_for_visual_boundary(
            5,
            UiTextVisualBoundaryBias::TrailingPrevious,
            line.source_range.end,
        ),
        UiTextCaret {
            offset: 6,
            affinity: UiTextCaretAffinity::Upstream,
        }
    );
}

#[test]
fn non_isomorphic_multi_grapheme_caret_snaps_to_whole_run_edges() {
    let line = UiResolvedTextLine {
        text: "..".to_string(),
        frame: UiFrame::new(10.0, 20.0, 20.0, 12.0),
        source_range: UiTextRange { start: 4, end: 8 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 20.0,
        glyph_advances: vec![10.0, 10.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![run("..", 4, 8, 0, 2, UiTextDirection::LeftToRight)],
        ellipsized: true,
    };
    let map = UiTextLineSourceMap::new(&line);

    assert_eq!(
        map.visual_offset_for_caret(&UiTextCaret {
            offset: 6,
            affinity: UiTextCaretAffinity::Upstream,
        }),
        0
    );
    assert_eq!(
        map.visual_offset_for_caret(&UiTextCaret {
            offset: 6,
            affinity: UiTextCaretAffinity::Downstream,
        }),
        2
    );
}

#[test]
fn combining_grapheme_split_across_runs_keeps_one_cluster_and_legal_caret_edges() {
    let line = UiResolvedTextLine {
        text: "a\u{0301}".to_string(),
        frame: UiFrame::new(10.0, 20.0, 20.0, 12.0),
        source_range: UiTextRange { start: 0, end: 3 },
        visual_range: UiTextRange { start: 0, end: 3 },
        measured_width: 20.0,
        glyph_advances: vec![20.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![
            run("a", 0, 1, 0, 1, UiTextDirection::LeftToRight),
            run("\u{0301}", 1, 3, 1, 3, UiTextDirection::LeftToRight),
        ],
        ellipsized: false,
    };
    let map = UiTextLineSourceMap::new(&line);

    assert_eq!(map.cluster_count(), 1);
    assert_eq!(map.advance_to_visual_offset(3), 20.0);
    assert_eq!(
        map.caret_for_visual_boundary(
            0,
            UiTextVisualBoundaryBias::LeadingCurrent,
            line.source_range.start,
        ),
        UiTextCaret {
            offset: 0,
            affinity: UiTextCaretAffinity::Downstream,
        }
    );
    assert_eq!(
        map.caret_for_visual_boundary(
            1,
            UiTextVisualBoundaryBias::LeadingCurrent,
            line.source_range.end,
        ),
        UiTextCaret {
            offset: 3,
            affinity: UiTextCaretAffinity::Downstream,
        }
    );
    assert_eq!(
        map.caret_for_visual_boundary(
            1,
            UiTextVisualBoundaryBias::TrailingPrevious,
            line.source_range.end,
        ),
        UiTextCaret {
            offset: 3,
            affinity: UiTextCaretAffinity::Upstream,
        }
    );
}

fn run(
    text: &str,
    source_start: usize,
    source_end: usize,
    visual_start: usize,
    visual_end: usize,
    direction: UiTextDirection,
) -> UiResolvedTextRun {
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
        direction,
    }
}
