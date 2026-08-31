use crate::ui::layout::UiFrame;

use super::super::{UiResolvedTextLine, UiResolvedTextRun};
use super::{
    UiResolvedTextLayout, UiTextDirection, UiTextRange, UiTextRunKind,
    text_paint_runs_from_resolved_layout,
};

#[test]
fn resolved_paint_projection_rejects_the_complete_batch_when_one_run_has_invalid_advances() {
    let layout = UiResolvedTextLayout {
        lines: vec![
            resolved_line("a", 0, vec![1.0], vec![resolved_run("a", 0)]),
            resolved_line("b", 1, Vec::new(), vec![resolved_run("b", 1)]),
        ],
        ..UiResolvedTextLayout::default()
    };

    let paint_runs = project(&layout);

    assert!(paint_runs.is_empty());
}

#[test]
fn resolved_paint_projection_keeps_valid_nonempty_runs_and_ignores_empty_runs() {
    let layout = UiResolvedTextLayout {
        lines: vec![resolved_line(
            "a",
            0,
            vec![3.0],
            vec![resolved_run("", 0), resolved_run("a", 0)],
        )],
        ..UiResolvedTextLayout::default()
    };

    let paint_runs = project(&layout);

    assert_eq!(paint_runs.len(), 1);
    assert_eq!(paint_runs[0].text, "a");
    assert_eq!(paint_runs[0].frame, UiFrame::new(0.0, 0.0, 3.0, 12.0));
}

#[test]
fn resolved_paint_projection_rejects_invalid_visual_utf8_ranges() {
    for visual_range in [
        UiTextRange { start: 1, end: 3 },
        UiTextRange { start: 0, end: 4 },
        UiTextRange { start: 3, end: 0 },
    ] {
        let mut run = resolved_run("語", 0);
        run.visual_range = visual_range;
        let layout = UiResolvedTextLayout {
            lines: vec![resolved_line("語", 0, vec![3.0], vec![run])],
            ..UiResolvedTextLayout::default()
        };

        assert!(project(&layout).is_empty(), "accepted {visual_range:?}");
    }
}

#[test]
fn resolved_paint_projection_preserves_scalar_aligned_style_boundaries_inside_a_grapheme() {
    let mut base = resolved_run("a", 0);
    base.visual_range = UiTextRange { start: 0, end: 1 };
    let mut combining = resolved_run("\u{0301}", 1);
    combining.visual_range = UiTextRange { start: 1, end: 3 };
    let layout = UiResolvedTextLayout {
        lines: vec![resolved_line(
            "a\u{0301}",
            0,
            vec![4.0],
            vec![base, combining],
        )],
        ..UiResolvedTextLayout::default()
    };

    let paint_runs = project(&layout);

    assert_eq!(paint_runs.len(), 2);
    assert_eq!(paint_runs[0].frame, UiFrame::new(0.0, 0.0, 4.0, 12.0));
    assert_eq!(paint_runs[1].frame, paint_runs[0].frame);
}

#[test]
fn resolved_paint_projection_rejects_run_text_that_disagrees_with_visual_slice() {
    let layout = UiResolvedTextLayout {
        lines: vec![resolved_line("a", 0, vec![3.0], vec![resolved_run("x", 0)])],
        ..UiResolvedTextLayout::default()
    };

    assert!(project(&layout).is_empty());
}

#[test]
fn resolved_paint_projection_rejects_non_contiguous_visual_runs() {
    let layout = UiResolvedTextLayout {
        lines: vec![resolved_line(
            "aa",
            0,
            vec![3.0, 3.0],
            vec![resolved_run("a", 0), resolved_run("a", 1)],
        )],
        ..UiResolvedTextLayout::default()
    };

    assert!(project(&layout).is_empty());
}

fn project(layout: &UiResolvedTextLayout) -> Vec<super::UiTextPaintRun> {
    let no_string = None;
    text_paint_runs_from_resolved_layout(
        layout, &no_string, &no_string, &no_string, 400, 10.0, 12.0,
    )
}

fn resolved_line(
    text: &str,
    source_start: usize,
    glyph_advances: Vec<f32>,
    runs: Vec<UiResolvedTextRun>,
) -> UiResolvedTextLine {
    let source_end = source_start + text.len();
    UiResolvedTextLine {
        text: text.to_owned(),
        frame: UiFrame::new(0.0, source_start as f32 * 12.0, 3.0, 12.0),
        placement_frame: UiFrame::new(0.0, source_start as f32 * 12.0, 3.0, 12.0),
        source_range: UiTextRange {
            start: source_start,
            end: source_end,
        },
        visual_range: UiTextRange {
            start: 0,
            end: text.len(),
        },
        measured_width: 3.0,
        glyph_advances,
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs,
        ellipsized: false,
    }
}

fn resolved_run(text: &str, source_start: usize) -> UiResolvedTextRun {
    UiResolvedTextRun {
        kind: UiTextRunKind::Plain,
        text: text.to_owned(),
        source_range: UiTextRange {
            start: source_start,
            end: source_start + text.len(),
        },
        visual_range: UiTextRange {
            start: 0,
            end: text.len(),
        },
        direction: UiTextDirection::LeftToRight,
    }
}
