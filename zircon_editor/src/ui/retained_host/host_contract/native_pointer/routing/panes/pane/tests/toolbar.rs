use super::*;
use crate::ui::retained_host::callback_dispatch::BuiltinViewportToolbarTemplateBridge;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn actual_scaled_toolbar_routes_source_hover_identity_and_preserves_command_id() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();
    bridge.admit_layout_context(1.5, 1);
    bridge.recompute_layout(UiSize::new(960.0, 42.0)).unwrap();
    let mut pane = PaneData {
        kind: "Scene".into(),
        show_toolbar: true,
        ..Default::default()
    };
    pane.viewport.toolbar_surface_key = "document:route".into();
    pane.viewport.toolbar_template_nodes = bridge
        .paint_nodes_for_size(UiSize::new(960.0, 42.0))
        .unwrap();
    pane.viewport.toolbar_surface_frame = Some(bridge.surface_frame_for_projection_controls(
        "document:route",
        UiSize::new(960.0, 42.0),
        |id| {
            Some(
                if id == "ActivateSceneMode" {
                    "mode.select"
                } else {
                    id
                }
                .into(),
            )
        },
    ));
    let toolbar = FrameRect {
        x: 240.0,
        y: 160.0,
        width: 960.0,
        height: 42.0,
    };
    let pointer = route_viewport_toolbar(&pane, &toolbar, 261.0, 181.0, Some("document:route"));
    match pointer.target {
        PanePointerTarget::ViewportToolbar {
            control_id,
            source_control_id,
            control_frame,
            ..
        } => {
            assert_eq!(control_id, Some("mode.select"));
            assert_eq!(
                source_control_id.as_deref(),
                Some("document:route::ActivateSceneMode")
            );
            assert_eq!(
                control_frame,
                FrameRect {
                    x: 240.0,
                    y: 160.0,
                    width: 42.0,
                    height: 42.0
                }
            );
            let ui = crate::ui::retained_host::UiHostWindow::new().unwrap();
            ui.set_hovered_template_node_for_pointer_move(
                source_control_id.as_deref().unwrap(),
                &control_frame,
            );
            ui.set_template_button_pointer_state(
                source_control_id.as_deref(),
                Some(&control_frame),
                true,
            );
            let state = ui.get_pane_interaction_state();
            assert_eq!(
                state.hovered_template_control_id,
                state.focused_template_control_id
            );
            assert_eq!(
                state.focused_template_control_id,
                state.pressed_template_control_id
            );
            ui.set_template_button_pointer_state(
                source_control_id.as_deref(),
                Some(&control_frame),
                false,
            );
            assert!(ui
                .get_pane_interaction_state()
                .pressed_template_control_id
                .is_empty());
        }
        _ => panic!("actual source toolbar control must keep its native route"),
    }
}
