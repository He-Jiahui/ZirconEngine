use crate::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::UiRgbaColor,
    surface::{
        UiPaintPayload, UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiResolvedTextBox,
        UiResolvedTextLayout, UiTextPaintDecorationKind, UiTextRange,
    },
};

#[test]
fn ui_text_paint_projects_resolved_table_cell_boxes_to_ordered_decorations() {
    let cell_frame = UiFrame::new(12.0, 18.0, 96.0, 34.0);
    let cell_range = UiTextRange { start: 0, end: 4 };
    let command = UiRenderCommand {
        node_id: UiNodeId::new(12),
        kind: UiRenderCommandKind::Text,
        frame: cell_frame,
        clip_frame: None,
        z_index: 2,
        style: UiResolvedStyle::default(),
        text_layout: Some(UiResolvedTextLayout {
            source_range: cell_range,
            boxes: vec![UiResolvedTextBox {
                range: cell_range,
                frame: cell_frame,
                background_color: Some(UiRgbaColor::from_u8(0x12, 0x20, 0x2C, 0x80)),
                border_color: Some(UiRgbaColor::from_u8(0x73, 0xD7, 0xFF, 0xFF)),
                border_width: 1.0,
            }],
            ..UiResolvedTextLayout::default()
        }),
        text: Some("cell".to_string()),
        image: None,
        opacity: 1.0,
    };

    let element = command.to_paint_element(0);
    let UiPaintPayload::Text { text } = element.payload else {
        panic!("expected text payload");
    };
    assert_eq!(text.decorations.len(), 2);
    assert_eq!(
        text.decorations[0].kind,
        UiTextPaintDecorationKind::TableCellBackground
    );
    assert_eq!(text.decorations[0].range, cell_range);
    assert_eq!(text.decorations[0].frame, cell_frame);
    assert_eq!(text.decorations[0].color, "#12202C80");
    assert_eq!(
        text.decorations[1].kind,
        UiTextPaintDecorationKind::TableCellBorder
    );
    assert_eq!(text.decorations[1].color, "#73D7FFFF");
    assert_eq!(text.decorations[1].thickness, 1.0);
}
