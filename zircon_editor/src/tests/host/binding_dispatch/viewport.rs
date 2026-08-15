use zircon_runtime_interface::math::UVec2;

use super::support;
use crate::core::editing::intent::EditorIntent;
use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, TransformHandleKind, TransformSpace, ViewOrientation,
};
use crate::ui::binding::{
    EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, ViewportCommand,
};
use crate::ui::binding_dispatch::apply_viewport_binding;

#[test]
fn viewport_binding_applies_resize_command_to_editor_state() {
    let mut state = support::test_state();
    let binding = EditorUiBinding::new(
        "SceneView",
        "ViewportSurface",
        EditorUiEventKind::Resize,
        EditorUiBindingPayload::viewport_command(ViewportCommand::Resized {
            width: 1024,
            height: 768,
        }),
    );

    let feedback = apply_viewport_binding(&mut state, &binding).unwrap();

    assert_eq!(state.viewport_state().size, UVec2::new(1024, 768));
    assert!(!feedback.camera_updated);
}

#[test]
fn viewport_binding_applies_toolbar_commands_to_scene_viewport_state() {
    let mut state = support::test_state();

    let commands = [
        ViewportCommand::ActivateSceneMode(SceneModeActivation::Transform(
            TransformHandleKind::Rotate,
        )),
        ViewportCommand::SetTransformSpace(TransformSpace::Global),
        ViewportCommand::SetProjectionMode(ProjectionMode::Orthographic),
        ViewportCommand::AlignView(ViewOrientation::NegZ),
        ViewportCommand::SetDisplayMode(DisplayMode::WireOnly),
        ViewportCommand::SetGridMode(GridMode::VisibleAndSnap),
        ViewportCommand::SetTranslateSnap(2.0),
        ViewportCommand::SetRotateSnapDegrees(30.0),
        ViewportCommand::SetScaleSnap(0.25),
        ViewportCommand::SetPreviewLighting(false),
        ViewportCommand::SetPreviewSkybox(false),
        ViewportCommand::SetGizmosEnabled(false),
    ];

    for command in commands {
        let binding = EditorUiBinding::new(
            "SceneView",
            "ViewportToolbar",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::viewport_command(command),
        );
        let _ = apply_viewport_binding(&mut state, &binding).unwrap();
    }

    let settings = state.scene_viewport_settings();
    assert_eq!(
        settings.mode,
        SceneModeActivation::Transform(TransformHandleKind::Rotate)
    );
    assert_eq!(settings.transform_space, TransformSpace::Global);
    assert_eq!(settings.projection_mode, ProjectionMode::Orthographic);
    assert_eq!(settings.view_orientation, ViewOrientation::NegZ);
    assert_eq!(settings.display_mode, DisplayMode::WireOnly);
    assert_eq!(settings.grid_mode, GridMode::VisibleAndSnap);
    assert_eq!(settings.translate_step, 2.0);
    assert_eq!(settings.rotate_step_deg, 30.0);
    assert_eq!(settings.scale_step, 0.25);
    assert!(!settings.preview_lighting);
    assert!(!settings.preview_skybox);
    assert!(!settings.gizmos_enabled);
}

#[test]
fn viewport_toggle_bindings_flow_into_render_packet() {
    let mut state = support::test_state();
    let camera = state
        .world
        .with_world(|scene: &zircon_runtime::scene::Scene| scene.active_camera());

    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();

    for command in [
        ViewportCommand::SetGizmosEnabled(false),
        ViewportCommand::SetDisplayMode(DisplayMode::WireOverlay),
        ViewportCommand::SetGridMode(GridMode::VisibleNoSnap),
        ViewportCommand::SetPreviewLighting(false),
        ViewportCommand::SetPreviewSkybox(false),
    ] {
        let binding = EditorUiBinding::new(
            "SceneView",
            "ViewportToolbar",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::viewport_command(command),
        );
        let _ = apply_viewport_binding(&mut state, &binding).unwrap();
    }

    let packet = state.render_snapshot().expect("render packet");

    assert_eq!(packet.overlays.display_mode, DisplayMode::WireOverlay);
    assert_eq!(
        packet.overlays.grid.as_ref().map(|grid| grid.snap_enabled),
        Some(false)
    );
    assert!(packet.overlays.scene_gizmos.is_empty());
    assert!(!packet.preview.lighting_enabled);
    assert!(!packet.preview.skybox_enabled);
}

#[test]
fn gizmos_toggle_keeps_transform_handles_for_selected_camera() {
    let mut state = support::test_state();
    let camera = state
        .world
        .with_world(|scene: &zircon_runtime::scene::Scene| scene.active_camera());
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();

    for command in [
        ViewportCommand::ActivateSceneMode(SceneModeActivation::Transform(
            TransformHandleKind::Move,
        )),
        ViewportCommand::SetGizmosEnabled(false),
    ] {
        let binding = EditorUiBinding::new(
            "SceneView",
            "ViewportToolbar",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::viewport_command(command),
        );
        let _ = apply_viewport_binding(&mut state, &binding).unwrap();
    }

    let packet = state.render_snapshot().expect("render packet");

    assert!(packet.overlays.scene_gizmos.is_empty());
    assert_eq!(packet.overlays.selection_anchors.len(), 1);
    assert_eq!(packet.overlays.handles.len(), 1);
}

#[test]
fn drag_tool_keeps_renderable_highlight_without_handles() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();

    let binding = EditorUiBinding::new(
        "SceneView",
        "ViewportToolbar",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::viewport_command(ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Select,
        )),
    );
    let _ = apply_viewport_binding(&mut state, &binding).unwrap();

    let packet = state.render_snapshot().expect("render packet");

    assert_eq!(packet.overlays.selection.len(), 1);
    assert!(packet.overlays.handles.is_empty());
}
