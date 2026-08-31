use crate::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{
        UiPaintPayload, UiRenderCommand, UiRenderCommandKind, UiResolvedStyle,
        UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextDirection, UiTextRange,
        UiTextRunKind, UiTextShapeArtifact, UiTextWritingMode,
    },
};

#[test]
fn resolved_layout_paint_reports_canonical_glyph_artifact_unavailable() {
    let command = UiRenderCommand {
        node_id: UiNodeId::new(9002),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 120.0, 32.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            font_size: 16.0,
            line_height: 20.0,
            ..UiResolvedStyle::default()
        },
        text_layout: Some(UiResolvedTextLayout {
            direction: UiTextDirection::LeftToRight,
            font_size: 16.0,
            line_height: 20.0,
            source_range: UiTextRange { start: 0, end: 2 },
            lines: vec![UiResolvedTextLine {
                text: "Wi".to_string(),
                placement_frame: UiFrame::default(),
                frame: UiFrame::new(4.0, 6.0, 18.0, 20.0),
                source_range: UiTextRange { start: 0, end: 2 },
                visual_range: UiTextRange { start: 0, end: 2 },
                measured_width: 18.0,
                glyph_advances: vec![14.0, 4.0],
                baseline: 14.0,
                direction: UiTextDirection::LeftToRight,
                runs: vec![UiResolvedTextRun {
                    kind: UiTextRunKind::Strong,
                    text: "Wi".to_string(),
                    source_range: UiTextRange { start: 0, end: 2 },
                    visual_range: UiTextRange { start: 0, end: 2 },
                    direction: UiTextDirection::LeftToRight,
                }],
                ellipsized: false,
            }],
            ..UiResolvedTextLayout::default()
        }),
        text: Some("Wi".to_string()),
        image: None,
        opacity: 1.0,
    };

    let paint = text_paint(command);

    assert_eq!(paint.shaped, UiTextShapeArtifact::Unavailable);
    assert_eq!(paint.runs.len(), 1);
    assert_eq!(paint.runs[0].frame, UiFrame::new(4.0, 6.0, 18.0, 20.0));
    assert!(paint.runs[0].style.strong);
}

#[test]
fn resolved_layout_paint_uses_exact_horizontal_run_advances() {
    let command = UiRenderCommand {
        node_id: UiNodeId::new(9003),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 120.0, 32.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle::default(),
        text_layout: Some(layout_with_runs(UiTextWritingMode::HorizontalTb)),
        text: Some("A\u{8a9e}B".to_string()),
        image: None,
        opacity: 1.0,
    };

    let paint = text_paint(command);

    assert_eq!(paint.runs.len(), 2);
    assert_eq!(paint.runs[0].frame, UiFrame::new(10.0, 6.0, 3.0, 20.0));
    assert_eq!(paint.runs[1].frame, UiFrame::new(13.0, 6.0, 16.0, 20.0));
    assert!(paint.runs[1].style.strong);
}

#[test]
fn resolved_layout_paint_uses_exact_vertical_run_advances_and_rejects_incomplete_data() {
    let layout = layout_with_runs(UiTextWritingMode::VerticalRl);
    let paint = text_paint(text_command(9004, layout.clone()));

    assert_eq!(paint.runs.len(), 2);
    assert_eq!(paint.runs[0].frame, UiFrame::new(20.0, 4.0, 12.0, 3.0));
    assert_eq!(paint.runs[1].frame, UiFrame::new(20.0, 7.0, 12.0, 16.0));
    assert!(paint.runs[1].style.emphasis);

    let mut incomplete_layout = layout;
    incomplete_layout.lines[0].glyph_advances.pop();
    assert!(text_paint(text_command(9005, incomplete_layout))
        .runs
        .is_empty());
}

fn layout_with_runs(writing_mode: UiTextWritingMode) -> UiResolvedTextLayout {
    let vertical = matches!(writing_mode, UiTextWritingMode::VerticalRl);
    UiResolvedTextLayout {
        direction: UiTextDirection::LeftToRight,
        font_size: 16.0,
        line_height: 20.0,
        writing_mode,
        source_range: UiTextRange { start: 0, end: 5 },
        lines: vec![UiResolvedTextLine {
            text: "A\u{8a9e}B".to_string(),
            placement_frame: UiFrame::default(),
            frame: if vertical {
                UiFrame::new(20.0, 4.0, 12.0, 19.0)
            } else {
                UiFrame::new(10.0, 6.0, 19.0, 20.0)
            },
            source_range: UiTextRange { start: 0, end: 5 },
            visual_range: UiTextRange { start: 0, end: 5 },
            measured_width: 19.0,
            glyph_advances: vec![3.0, 11.0, 5.0],
            baseline: if vertical { 9.0 } else { 14.0 },
            direction: UiTextDirection::LeftToRight,
            runs: vec![
                UiResolvedTextRun {
                    kind: UiTextRunKind::Plain,
                    text: "A".to_string(),
                    source_range: UiTextRange { start: 0, end: 1 },
                    visual_range: UiTextRange { start: 0, end: 1 },
                    direction: UiTextDirection::LeftToRight,
                },
                UiResolvedTextRun {
                    kind: if vertical {
                        UiTextRunKind::Emphasis
                    } else {
                        UiTextRunKind::Strong
                    },
                    text: "\u{8a9e}B".to_string(),
                    source_range: UiTextRange { start: 1, end: 5 },
                    visual_range: UiTextRange { start: 1, end: 5 },
                    direction: UiTextDirection::LeftToRight,
                },
            ],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    }
}

fn text_command(node_id: u64, layout: UiResolvedTextLayout) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 120.0, 64.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            text_writing_mode: layout.writing_mode,
            ..UiResolvedStyle::default()
        },
        text_layout: Some(layout),
        text: Some("A\u{8a9e}B".to_string()),
        image: None,
        opacity: 1.0,
    }
}

fn text_paint(command: UiRenderCommand) -> crate::ui::surface::UiTextPaint {
    command
        .to_paint_elements(0)
        .into_iter()
        .find_map(|element| match element.payload {
            UiPaintPayload::Text { text } => Some(text),
            _ => None,
        })
        .expect("text paint payload")
}
