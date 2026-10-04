use super::*;
use crate::ui::surface::measure_text_source_range_width;
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextLine, UiResolvedTextRun, UiTextCaretAffinity, UiTextDirection, UiTextRunKind,
};

#[test]
fn surface_text_caret_frame_uses_source_range_metrics() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = "Wi";
    let layout = layout_with_advances(text, vec![1.0, 1.0]);
    let caret = UiTextCaret {
        offset: "W".len(),
        affinity: UiTextCaretAffinity::Downstream,
    };
    let expected_prefix = measure_text_source_range_width(
        text,
        &style,
        UiTextRange {
            start: 0,
            end: "W".len(),
        },
    );

    let frame = text_caret_frame_for_layout(&layout, &caret, text, &style).expect("caret frame");
    let selection = text_range_frames_for_layout(
        &layout,
        UiTextRange {
            start: 0,
            end: "W".len(),
        },
        text,
        &style,
    );

    assert!((frame.x - (10.0 + expected_prefix)).abs() < 0.1);
    assert!((frame.x - 11.0).abs() > 0.5);
    assert_eq!(selection.len(), 1);
    assert!((selection[0].width - expected_prefix).abs() < 0.1);
}

#[test]
fn surface_text_caret_frame_keeps_tab_resolved_advances() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = "a\tb";
    let layout = layout_with_advances(text, vec![6.0, 18.0, 6.0]);
    let caret = UiTextCaret {
        offset: 2,
        affinity: UiTextCaretAffinity::Downstream,
    };

    let frame = text_caret_frame_for_layout(&layout, &caret, text, &style).expect("caret frame");

    assert_eq!(frame, UiFrame::new(34.0, 20.0, 1.0, 12.0));
}

#[test]
fn surface_text_caret_frame_rejects_unresolved_auto_direction_source_metrics() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    };
    let text = "Wi";
    let mut layout = layout_with_advances(text, vec![2.0, 20.0]);
    layout.direction = UiTextDirection::Auto;
    let caret = UiTextCaret {
        offset: "W".len(),
        affinity: UiTextCaretAffinity::Downstream,
    };

    let frame = text_caret_frame_for_layout(&layout, &caret, text, &style).expect("caret frame");
    let selection = text_range_frames_for_layout(
        &layout,
        UiTextRange {
            start: 0,
            end: "W".len(),
        },
        text,
        &style,
    );

    assert_eq!(frame, UiFrame::new(12.0, 20.0, 1.0, 12.0));
    assert_eq!(selection, vec![UiFrame::new(10.0, 20.0, 2.0, 12.0)]);
}

fn layout_with_advances(text: &str, glyph_advances: Vec<f32>) -> UiResolvedTextLayout {
    UiResolvedTextLayout {
        font_size: 10.0,
        line_height: 12.0,
        source_range: UiTextRange {
            start: 0,
            end: text.len(),
        },
        direction: UiTextDirection::LeftToRight,
        lines: vec![UiResolvedTextLine {
            text: text.to_string(),
            placement_frame: UiFrame::default(),
            frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
            source_range: UiTextRange {
                start: 0,
                end: text.len(),
            },
            visual_range: UiTextRange {
                start: 0,
                end: text.len(),
            },
            measured_width: 30.0,
            glyph_advances,
            baseline: 9.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: text.to_string(),
                source_range: UiTextRange {
                    start: 0,
                    end: text.len(),
                },
                visual_range: UiTextRange {
                    start: 0,
                    end: text.len(),
                },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    }
}
