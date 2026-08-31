use super::*;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextLayout, UiResolvedTextRun, UiTextDirection, UiTextRunKind, UiTextWritingMode,
};

#[test]
fn glyph_artifact_rejects_a_source_that_cannot_own_the_layout_range() {
    let source = "short";
    let source_range = UiTextRange { start: 8, end: 16 };
    let layout = plain_layout("detached", source_range);
    let mut provider = SharedTextLayoutSession::new();

    assert!(matches!(
        build_resolved_text_glyph_artifact(source, &style(), &layout, &mut provider),
        TextShapingOutcome::Failed(TextLayoutError::LayoutFailed)
    ));
    assert_eq!(provider.cache_report().insert_count, 0);
}

#[test]
fn glyph_artifact_rejects_a_line_range_outside_its_layout_owner() {
    let source = "0123456789abcdef";
    let layout_source_range = UiTextRange { start: 4, end: 12 };
    let mut layout = plain_layout("89abcdef", UiTextRange { start: 8, end: 16 });
    layout.source_range = layout_source_range;
    let mut provider = SharedTextLayoutSession::new();

    assert!(matches!(
        build_resolved_text_glyph_artifact(source, &style(), &layout, &mut provider),
        TextShapingOutcome::Failed(TextLayoutError::LayoutFailed)
    ));
    assert_eq!(provider.cache_report().insert_count, 0);
}

#[test]
fn glyph_artifact_rejects_a_line_range_that_splits_a_utf8_scalar() {
    let source = "é";
    let mut layout = plain_layout("?", UiTextRange { start: 1, end: 2 });
    layout.source_range = UiTextRange {
        start: 0,
        end: source.len(),
    };
    let mut provider = SharedTextLayoutSession::new();

    assert!(matches!(
        build_resolved_text_glyph_artifact(source, &style(), &layout, &mut provider),
        TextShapingOutcome::Failed(TextLayoutError::LayoutFailed)
    ));
    assert_eq!(provider.cache_report().insert_count, 0);
}

#[test]
fn glyph_artifact_rejects_a_non_empty_run_range_that_splits_a_utf8_scalar() {
    let source = "é";
    let source_range = UiTextRange {
        start: 0,
        end: source.len(),
    };
    let mut layout = plain_layout(source, source_range);
    layout.lines[0].runs[0].source_range = UiTextRange { start: 1, end: 2 };
    let mut provider = SharedTextLayoutSession::new();

    assert!(matches!(
        build_resolved_text_glyph_artifact(source, &style(), &layout, &mut provider),
        TextShapingOutcome::Failed(TextLayoutError::LayoutFailed)
    ));
    assert_eq!(provider.cache_report().insert_count, 0);
}

#[test]
fn glyph_artifact_rejects_empty_virtual_ranges_at_non_utf8_anchors() {
    let source = "é";
    let source_range = UiTextRange {
        start: 0,
        end: source.len(),
    };
    let mut layout = plain_layout(source, source_range);
    layout.lines[0].runs.push(UiResolvedTextRun {
        kind: UiTextRunKind::Plain,
        text: "...".to_string(),
        source_range: UiTextRange { start: 1, end: 1 },
        visual_range: UiTextRange { start: 2, end: 5 },
        direction: UiTextDirection::LeftToRight,
    });

    assert!(!artifact_line_source_ranges_are_sliceable(
        source,
        0,
        &layout.lines[0]
    ));

    layout.lines[0].runs[1].source_range = UiTextRange {
        start: source.len(),
        end: source.len(),
    };
    assert!(artifact_line_source_ranges_are_sliceable(
        source,
        0,
        &layout.lines[0]
    ));
}

#[test]
fn glyph_artifact_keeps_a_visual_only_line_as_ready_without_an_artifact() {
    let source = "hidden";
    let source_range = UiTextRange {
        start: 0,
        end: source.len(),
    };
    let mut layout = plain_layout("…", source_range);
    layout.lines[0].runs.clear();
    layout.lines[0].ellipsized = true;
    let mut provider = SharedTextLayoutSession::new();

    assert!(matches!(
        build_resolved_text_glyph_artifact(source, &style(), &layout, &mut provider),
        TextShapingOutcome::Ready(None)
    ));
    assert_eq!(provider.cache_report().insert_count, 0);
}

fn plain_layout(text: &str, source_range: UiTextRange) -> UiResolvedTextLayout {
    let style = style();
    let line = UiResolvedTextLine {
        text: text.to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 64.0, style.line_height),
        source_range,
        visual_range: UiTextRange {
            start: 0,
            end: text.len(),
        },
        measured_width: 64.0,
        glyph_advances: vec![8.0; text.chars().count()],
        baseline: 16.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: text.to_string(),
            source_range,
            visual_range: UiTextRange {
                start: 0,
                end: text.len(),
            },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };
    UiResolvedTextLayout {
        writing_mode: UiTextWritingMode::HorizontalTb,
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width: line.measured_width,
        measured_height: line.frame.height,
        source_range,
        lines: vec![line],
        ..UiResolvedTextLayout::default()
    }
}

fn style() -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 16.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    }
}
