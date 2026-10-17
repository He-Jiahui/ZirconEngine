use super::*;
use crate::scene::viewport::handles::{
    handle_build_context::HandleSelection, handle_drag_session::HandleDragSession,
    handle_tool::HandleTool, move_handle_tool::MoveHandleTool,
    rotate_handle_tool::RotateHandleTool, scale_handle_tool::ScaleHandleTool,
};
use crate::scene::viewport::{
    GridMode, SceneViewportSettings, SceneViewportSnapSteps, ViewportCameraSnapshot,
};
use zircon_runtime_interface::math::{Transform, Vec2};

#[test]
fn unreal_each_real_handle_drag_consumes_its_own_snap_boolean_with_grid_hidden() {
    let mut settings = SceneViewportSettings::default();
    settings.grid_mode = GridMode::Hidden;
    let camera = ViewportCameraSnapshot::default();
    let ctx = HandlePickContext {
        selected: Some(HandleSelection {
            entity: 7,
            transform: Transform::default(),
        }),
        settings: &settings,
        camera: &camera,
        cursor: Vec2::ZERO,
        snap_steps: SceneViewportSnapSteps {
            translate_step: 2.0,
            rotate_step_deg: 30.0,
            scale_step: 0.25,
            translate_enabled: true,
            rotate_enabled: false,
            scale_enabled: true,
        },
    };
    let tools: [&dyn HandleTool; 3] = [&MoveHandleTool, &RotateHandleTool, &ScaleHandleTool];
    for (index, tool) in tools.into_iter().enumerate() {
        let session = tool.begin_drag(&ctx, GizmoAxis::X).unwrap();
        let snap = match &session {
            HandleDragSession::Move(s)
            | HandleDragSession::Rotate(s)
            | HandleDragSession::Scale(s) => s,
        };
        assert_eq!(snap.snap_enabled, index != 1);
        assert_eq!(snap.translate_step, 2.0);
        assert_eq!(snap.rotate_step_radians, 30.0_f32.to_radians());
        assert_eq!(snap.scale_step, 0.25);
    }
    let no_selection = HandlePickContext {
        selected: None,
        ..ctx
    };
    assert!(MoveHandleTool
        .begin_drag(&no_selection, GizmoAxis::X)
        .is_none());
}
