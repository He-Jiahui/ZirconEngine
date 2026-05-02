use std::sync::Arc;

use crate::ui::slint_host::host_contract::WorldSpaceUiSurfaceSubmission;
use crate::ui::slint_host::viewport::tests::fake_render_framework::FakeRenderFramework;
use crate::ui::slint_host::viewport::tests::test_extract::test_extract;
use crate::ui::slint_host::viewport::SlintViewportController;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiPointerEventKind, UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList,
    UiResolvedStyle, UiTextAlign, UiTextRenderMode, UiTextWrap,
};

#[test]
fn controller_submits_shared_ui_overlay_through_render_framework() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract_with_ui(
            test_extract(),
            Some(test_ui_extract("Viewport HUD")),
            UVec2::new(300, 150),
        )
        .unwrap();

    let state = framework.state.lock().unwrap();
    assert_eq!(state.submitted_ui_command_counts, vec![1]);
}

#[test]
fn controller_submits_world_space_ui_surfaces_through_render_framework_ui_extract() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework.clone());

    controller.submit_world_space_ui_surfaces(vec![WorldSpaceUiSurfaceSubmission {
        surface_id: "viewport-main".to_string(),
        node_id: "world-panel".to_string(),
        control_id: "WorldPanel".to_string(),
        viewport_x: 10.0,
        viewport_y: 20.0,
        viewport_width: 200.0,
        viewport_height: 80.0,
        world_position: [1.0, 2.0, 3.0],
        world_rotation: [0.0, 0.0, 0.0],
        world_scale: [1.0, 1.0, 1.0],
        world_width: 2.0,
        world_height: 0.8,
        pixels_per_meter: 100.0,
        billboard: true,
        depth_test: true,
        render_order: 5,
        camera_target: "viewport-main".to_string(),
    }]);

    let submissions = controller.last_world_space_ui_surfaces();
    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].control_id, "WorldPanel");
    assert!(submissions[0].contains_viewport_point(16.0, 24.0));

    controller
        .submit_extract_with_ui(
            test_extract(),
            Some(test_ui_extract("Viewport HUD")),
            UVec2::new(320, 180),
        )
        .unwrap();

    let state = framework.state.lock().unwrap();
    assert_eq!(state.submitted_ui_command_counts, vec![2]);
    assert_eq!(
        state.submitted_ui_texts,
        vec![vec!["Viewport HUD".to_string(), "WorldPanel".to_string()]]
    );
}

#[test]
fn controller_routes_world_space_ui_pointer_hit_with_capture() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework);

    controller.submit_world_space_ui_surfaces(vec![WorldSpaceUiSurfaceSubmission {
        surface_id: "viewport-main".to_string(),
        node_id: "world-panel".to_string(),
        control_id: "WorldPanel".to_string(),
        viewport_x: 10.0,
        viewport_y: 20.0,
        viewport_width: 200.0,
        viewport_height: 80.0,
        world_position: [1.0, 2.0, 3.0],
        world_rotation: [0.0, 0.0, 0.0],
        world_scale: [1.0, 1.0, 1.0],
        world_width: 2.0,
        world_height: 0.8,
        pixels_per_meter: 100.0,
        billboard: true,
        depth_test: true,
        render_order: 5,
        camera_target: "viewport-main".to_string(),
    }]);

    let down = controller
        .route_world_space_ui_pointer_event(UiPointerEventKind::Down, 16.0, 24.0)
        .expect("down should hit world-space UI");
    assert_eq!(down.control_id, "WorldPanel");

    let captured_move = controller
        .route_world_space_ui_pointer_event(UiPointerEventKind::Move, 400.0, 400.0)
        .expect("move should stay captured by world-space UI");
    assert_eq!(captured_move.control_id, "WorldPanel");

    let released = controller
        .route_world_space_ui_pointer_event(UiPointerEventKind::Up, 400.0, 400.0)
        .expect("up should release the captured world-space UI target");
    assert_eq!(released.control_id, "WorldPanel");

    assert!(controller
        .route_world_space_ui_pointer_event(UiPointerEventKind::Move, 400.0, 400.0)
        .is_none());
}

fn test_ui_extract(text: &str) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new("editor.viewport.test"),
        list: UiRenderList {
            commands: vec![UiRenderCommand {
                node_id: UiNodeId::new(9),
                kind: UiRenderCommandKind::Quad,
                frame: UiFrame::new(8.0, 8.0, 160.0, 24.0),
                clip_frame: None,
                z_index: 0,
                style: UiResolvedStyle {
                    background_color: Some("#16202ccc".to_string()),
                    foreground_color: Some("#eef3ff".to_string()),
                    font: Some("res://fonts/default.font.toml".to_string()),
                    font_size: 13.0,
                    line_height: 16.0,
                    text_align: UiTextAlign::Center,
                    wrap: UiTextWrap::None,
                    text_render_mode: UiTextRenderMode::Auto,
                    ..UiResolvedStyle::default()
                },
                text_layout: None,
                text: Some(text.to_string()),
                image: None,
                opacity: 1.0,
            }],
        },
    }
}
