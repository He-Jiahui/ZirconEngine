use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiResolvedTextLayout, UiTextCaret, UiTextRange},
};

/// Returns a caret frame for an already-resolved text layout using shaped source-range metrics
/// whenever the layout line is simple enough to make source and visual ranges equivalent.
pub fn text_caret_frame_for_layout(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    source_text: &str,
    style: &UiResolvedStyle,
) -> Option<UiFrame> {
    crate::ui::text::caret_frame_for_text_layout_with_source_metrics(
        layout,
        caret,
        source_text,
        style,
    )
}

/// Returns source-range frames for an already-resolved text layout using shaped source-range
/// metrics when available, with the text geometry owner retaining fallback behavior.
pub fn text_range_frames_for_layout(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    source_text: &str,
    style: &UiResolvedStyle,
) -> Vec<UiFrame> {
    crate::ui::text::text_range_frames_for_text_layout_with_source_metrics(
        layout,
        range,
        source_text,
        style,
    )
}

#[cfg(test)]
mod tests {
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

        let frame =
            text_caret_frame_for_layout(&layout, &caret, text, &style).expect("caret frame");
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

        let frame =
            text_caret_frame_for_layout(&layout, &caret, text, &style).expect("caret frame");

        assert_eq!(frame, UiFrame::new(34.0, 20.0, 1.0, 12.0));
    }

    fn layout_with_advances(text: &str, glyph_advances: Vec<f32>) -> UiResolvedTextLayout {
        UiResolvedTextLayout {
            font_size: 10.0,
            line_height: 12.0,
            source_range: UiTextRange {
                start: 0,
                end: text.len(),
            },
            lines: vec![UiResolvedTextLine {
                text: text.to_string(),
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
}
