use crate::ui::slint_host::paint_runtime_render_commands_for_test;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{
        UiEditableTextState, UiRenderCommand, UiRenderCommandKind, UiResolvedStyle,
        UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextCaret,
        UiTextCaretAffinity, UiTextComposition, UiTextDirection, UiTextOverflow, UiTextRange,
        UiTextRunKind, UiTextSelection,
    },
};

#[test]
fn rust_owned_host_painter_draws_runtime_editable_text_decorations() {
    let bytes = paint_runtime_render_commands_for_test(80, 48, &[editable_text_command()]);

    assert_eq!(
        pixel(80, &bytes, 50, 22),
        [232, 238, 247, 255],
        "editable text caret should paint above text content"
    );
    assert_eq!(
        pixel(80, &bytes, 34, 31),
        [77, 137, 255, 255],
        "composition underline should paint as a visible runtime text decoration"
    );
    let selection = pixel(80, &bytes, 24, 20);
    assert!(
        selection[2] > selection[0] && selection[2] > 40,
        "selection decoration should paint a blue local highlight, got {selection:?}"
    );
}

fn editable_text_command() -> UiRenderCommand {
    let layout = UiResolvedTextLayout {
        direction: UiTextDirection::LeftToRight,
        overflow: UiTextOverflow::Clip,
        font_size: 10.0,
        line_height: 12.0,
        measured_width: 50.0,
        measured_height: 12.0,
        source_range: UiTextRange { start: 0, end: 5 },
        editable: Some(UiEditableTextState {
            text: "Hello".to_string(),
            caret: UiTextCaret {
                offset: 4,
                affinity: UiTextCaretAffinity::Downstream,
            },
            selection: Some(UiTextSelection {
                anchor: 1,
                focus: 3,
            }),
            composition: Some(UiTextComposition {
                range: UiTextRange { start: 2, end: 4 },
                text: "ll".to_string(),
                restore_text: None,
            }),
            read_only: false,
        }),
        lines: vec![UiResolvedTextLine {
            text: "Hello".to_string(),
            frame: UiFrame::new(10.0, 20.0, 50.0, 12.0),
            source_range: UiTextRange { start: 0, end: 5 },
            visual_range: UiTextRange { start: 0, end: 5 },
            measured_width: 50.0,
            baseline: 8.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "Hello".to_string(),
                source_range: UiTextRange { start: 0, end: 5 },
                visual_range: UiTextRange { start: 0, end: 5 },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    };
    UiRenderCommand {
        node_id: UiNodeId::new(91),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(10.0, 20.0, 50.0, 12.0),
        clip_frame: Some(UiFrame::new(0.0, 0.0, 80.0, 48.0)),
        z_index: 3,
        style: UiResolvedStyle {
            foreground_color: Some("#fedcba".to_string()),
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        },
        text_layout: Some(layout),
        text: Some("Hello".to_string()),
        image: None,
        opacity: 1.0,
    }
}

fn pixel(width: u32, bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
