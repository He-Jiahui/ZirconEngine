use super::*;

#[test]
fn screen_space_ui_plan_paints_table_cell_background_before_text_and_border_after_text() {
    let markup = "[table=1][cell border=#73D7FF bg=#12202C padding=8,4,12,6]cell[/cell][/table]";
    let style = UiResolvedStyle {
        foreground_color: Some("#ffffff".to_string()),
        font_size: 16.0,
        line_height: 20.0,
        wrap: UiTextWrap::None,
        text_render_mode: UiTextRenderMode::Native,
        rich_text_format: UiRichTextFormat::BbCodeV1,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 20.0, 180.0, 80.0);
    let layout = layout_text(markup, &style, frame, None);
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.rich-table-cell-box"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(13),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: None,
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(markup.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        UVec2::new(220, 120),
    );

    assert_eq!(plan.draws.len(), 1);
    assert_eq!(plan.draws[0].vertices, 0..6);
    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].text, "cell");
    assert_eq!(plan.post_text_draws.len(), 1);
    assert_eq!(plan.post_text_draws[0].vertices, 6..30);
}
