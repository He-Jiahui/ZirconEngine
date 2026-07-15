use super::*;
use crate::ui::{layout::UiFrame, surface::UiTextRunKind};

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
