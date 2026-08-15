use super::*;

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
fn artifact_projection_rejects_out_of_order_visual_runs() {
    let line = UiResolvedTextLine {
        text: "ab".to_string(),
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
