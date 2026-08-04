use super::*;

#[test]
fn screen_space_ui_plan_skips_a_command_with_a_clip_outside_the_viewport() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.clipping"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(21),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                    clip_frame: Some(UiFrame::new(240.0, 0.0, 20.0, 20.0)),
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#112233".to_string()),
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("clipped".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(200, 100),
    );

    assert!(plan.vertices.is_empty());
    assert!(plan.draws.is_empty());
    assert!(plan.post_text_draws.is_empty());
    assert!(plan.auto_texts.is_empty());
    assert!(plan.native_texts.is_empty());
    assert!(plan.sdf_texts.is_empty());
    assert!(plan.images.is_empty());
}

#[test]
fn screen_space_ui_plan_skips_a_command_when_its_clip_misses_the_command_frame() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.clipping"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(22),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                    clip_frame: Some(UiFrame::new(160.0, 0.0, 20.0, 20.0)),
                    z_index: 0,
                    style: UiResolvedStyle {
                        background_color: Some("#112233".to_string()),
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("clipped".to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
        },
        UVec2::new(200, 100),
    );

    assert!(plan.vertices.is_empty());
    assert!(plan.draws.is_empty());
    assert!(plan.native_texts.is_empty());
}

#[test]
fn screen_space_ui_plan_ignores_a_fully_clipped_quad_for_later_text_backgrounds() {
    let plan = plan_screen_space_ui_batches(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.ui.clipping"),
            list: UiRenderList {
                commands: vec![
                    UiRenderCommand {
                        node_id: UiNodeId::new(23),
                        kind: UiRenderCommandKind::Quad,
                        frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                        clip_frame: Some(UiFrame::new(240.0, 0.0, 20.0, 20.0)),
                        z_index: 0,
                        style: UiResolvedStyle {
                            background_color: Some("#112233".to_string()),
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: None,
                        image: None,
                        opacity: 1.0,
                    },
                    UiRenderCommand {
                        node_id: UiNodeId::new(24),
                        kind: UiRenderCommandKind::Text,
                        frame: UiFrame::new(8.0, 12.0, 120.0, 36.0),
                        clip_frame: None,
                        z_index: 1,
                        style: UiResolvedStyle {
                            foreground_color: Some("#ffffff".to_string()),
                            text_render_mode: UiTextRenderMode::Native,
                            ..UiResolvedStyle::default()
                        },
                        text_layout: None,
                        text: Some("visible".to_string()),
                        image: None,
                        opacity: 1.0,
                    },
                ],
            },
        },
        UVec2::new(200, 100),
    );

    assert_eq!(plan.native_texts.len(), 1);
    assert_eq!(plan.native_texts[0].background_color, None);
}
