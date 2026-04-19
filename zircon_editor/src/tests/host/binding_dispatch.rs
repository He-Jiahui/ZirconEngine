use crate::ui::{
    AssetCommand, DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
    InspectorFieldChange, SelectionCommand, ViewportCommand, WelcomeCommand,
};
use zircon_runtime::core::framework::animation::AnimationTrackPath;
use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use zircon_runtime::core::math::UVec2;
use zircon_runtime::ui::binding::UiBindingValue;

use crate::{
    apply_inspector_binding, apply_selection_binding, apply_viewport_binding,
    dispatch_animation_binding, dispatch_asset_binding, dispatch_docking_binding,
    dispatch_selection_binding, dispatch_welcome_binding, AnimationHostEvent, AssetHostEvent,
    LayoutCommand, SelectionHostEvent, WelcomeHostEvent,
};

#[test]
fn inspector_binding_applies_batch_changes_to_editor_state() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    state
        .apply_intent(crate::EditorIntent::SelectNode(cube))
        .unwrap();

    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            vec![
                InspectorFieldChange::new("name", UiBindingValue::string("Bound Cube")),
                InspectorFieldChange::new("parent", UiBindingValue::Null),
                InspectorFieldChange::new("transform.translation.x", UiBindingValue::Float(4.0)),
                InspectorFieldChange::new("transform.translation.y", UiBindingValue::Float(5.0)),
                InspectorFieldChange::new("transform.translation.z", UiBindingValue::Float(6.0)),
            ],
        ),
    );

    assert!(apply_inspector_binding(&mut state, &binding).unwrap());
    state.world.with_world(|scene: &zircon_runtime::scene::Scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, "Bound Cube");
        assert_eq!(
            node.transform.translation,
            zircon_runtime::core::math::Vec3::new(4.0, 5.0, 6.0)
        );
    });
}

#[test]
fn docking_binding_dispatches_into_layout_command() {
    let binding = EditorUiBinding::new(
        "HierarchyView",
        "AutoHideDrawer",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::SetDrawerMode {
            slot: "left_top".to_string(),
            mode: "AutoHide".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&binding).unwrap(),
        LayoutCommand::SetDrawerMode {
            slot: crate::ActivityDrawerSlot::LeftTop,
            mode: crate::ActivityDrawerMode::AutoHide,
        }
    );
}

#[test]
fn docking_preset_binding_dispatches_into_layout_command() {
    let save_binding = EditorUiBinding::new(
        "ToolWindow",
        "SavePreset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::SavePreset {
            name: "rider".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&save_binding).unwrap(),
        LayoutCommand::SavePreset {
            name: "rider".to_string(),
        }
    );

    let load_binding = EditorUiBinding::new(
        "ToolWindow",
        "LoadPreset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::LoadPreset {
            name: "rider".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&load_binding).unwrap(),
        LayoutCommand::LoadPreset {
            name: "rider".to_string(),
        }
    );
}

#[test]
fn docking_attach_binding_dispatches_into_layout_command() {
    let drawer_binding = EditorUiBinding::new(
        "ToolWindow",
        "DropToRight",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::AttachViewToDrawer {
            instance_id: "editor.project#1".to_string(),
            slot: "right_top".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&drawer_binding).unwrap(),
        LayoutCommand::AttachView {
            instance_id: crate::ViewInstanceId::new("editor.project#1"),
            target: crate::ViewHost::Drawer(crate::ActivityDrawerSlot::RightTop),
            anchor: None,
        }
    );

    let document_binding = EditorUiBinding::new(
        "DocumentTabs",
        "DropToDocument",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::AttachViewToDocument {
            instance_id: "editor.project#1".to_string(),
            page_id: "workbench".to_string(),
        }),
    );

    assert_eq!(
        dispatch_docking_binding(&document_binding).unwrap(),
        LayoutCommand::AttachView {
            instance_id: crate::ViewInstanceId::new("editor.project#1"),
            target: crate::ViewHost::Document(crate::MainPageId::workbench(), Vec::new()),
            anchor: None,
        }
    );
}

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
        ViewportCommand::SetTool(SceneViewportTool::Rotate),
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
    assert_eq!(settings.tool, SceneViewportTool::Rotate);
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
    let camera = state.world.with_world(|scene| scene.active_camera());

    state
        .apply_intent(crate::EditorIntent::SelectNode(camera))
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
    let camera = state.world.with_world(|scene| scene.active_camera());
    state
        .apply_intent(crate::EditorIntent::SelectNode(camera))
        .unwrap();

    for command in [
        ViewportCommand::SetTool(SceneViewportTool::Move),
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
    state
        .apply_intent(crate::EditorIntent::SelectNode(cube))
        .unwrap();

    let binding = EditorUiBinding::new(
        "SceneView",
        "ViewportToolbar",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(SceneViewportTool::Drag)),
    );
    let _ = apply_viewport_binding(&mut state, &binding).unwrap();

    let packet = state.render_snapshot().expect("render packet");

    assert_eq!(packet.overlays.selection.len(), 1);
    assert!(packet.overlays.handles.is_empty());
}

#[test]
fn animation_binding_dispatches_into_host_event() {
    let binding = EditorUiBinding::new(
        "AnimationClipEditorView",
        "AddFrameButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::position_of_track_and_frame("root/child:transform.translation", 24),
    );

    assert_eq!(
        dispatch_animation_binding(&binding).unwrap(),
        AnimationHostEvent::AddFrame {
            track_path: AnimationTrackPath::parse("root/child:transform.translation").unwrap(),
            frame: 24,
        }
    );
}

#[test]
fn selection_binding_dispatches_and_applies_scene_node_selection() {
    let mut state = support::test_state();
    let cube = support::cube_id(&state);
    let binding = EditorUiBinding::new(
        "HierarchyView",
        "SceneNodeSelect",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
            node_id: cube,
        }),
    );

    assert_eq!(
        dispatch_selection_binding(&binding).unwrap(),
        SelectionHostEvent::SelectSceneNode { node_id: cube }
    );
    assert!(apply_selection_binding(&mut state, &binding).unwrap());
    assert_eq!(state.viewport_controller.selected_node(), Some(cube));
}

#[test]
fn asset_binding_dispatches_into_host_event() {
    let binding = EditorUiBinding::new(
        "ProjectView",
        "OpenAsset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::asset_command(AssetCommand::OpenAsset {
            asset_path: "crate://prefabs/player.prefab".to_string(),
        }),
    );

    assert_eq!(
        dispatch_asset_binding(&binding).unwrap(),
        AssetHostEvent::OpenAsset {
            asset_path: "crate://prefabs/player.prefab".to_string(),
        }
    );
}

#[test]
fn asset_view_mode_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "AssetSurface",
        "SetViewMode",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::asset_command(AssetCommand::SetViewMode {
            surface: "browser".to_string(),
            view_mode: "thumbnail".to_string(),
        }),
    );

    assert_eq!(
        dispatch_asset_binding(&binding).unwrap(),
        AssetHostEvent::SetViewMode {
            surface: crate::EditorAssetSurface::Browser,
            view_mode: crate::EditorAssetViewMode::Thumbnail,
        }
    );
}

#[test]
fn asset_utility_tab_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "AssetSurface",
        "SetUtilityTab",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::asset_command(AssetCommand::SetUtilityTab {
            surface: "browser".to_string(),
            tab: "metadata".to_string(),
        }),
    );

    assert_eq!(
        dispatch_asset_binding(&binding).unwrap(),
        AssetHostEvent::SetUtilityTab {
            surface: crate::EditorAssetSurface::Browser,
            tab: crate::EditorAssetUtilityTab::Metadata,
        }
    );
}

#[test]
fn welcome_project_name_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "WelcomeSurface",
        "ProjectNameEdited",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::SetProjectName {
            value: "Sandbox".to_string(),
        }),
    );

    assert_eq!(
        dispatch_welcome_binding(&binding).unwrap(),
        WelcomeHostEvent::SetProjectName {
            value: "Sandbox".to_string(),
        }
    );
}

#[test]
fn welcome_open_recent_binding_dispatches_into_typed_host_event() {
    let binding = EditorUiBinding::new(
        "WelcomeSurface",
        "OpenRecentProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }),
    );

    assert_eq!(
        dispatch_welcome_binding(&binding).unwrap(),
        WelcomeHostEvent::OpenRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }
    );
}

mod support {
    use zircon_runtime::core::math::UVec2;
    use zircon_runtime::scene::DefaultLevelManager;
    use zircon_runtime::scene::components::NodeKind;
    use zircon_runtime::scene::NodeId;

    use crate::EditorState;

    pub fn test_state() -> EditorState {
        let manager = DefaultLevelManager::default();
        EditorState::with_default_selection(manager.create_default_level(), UVec2::new(1280, 720))
    }

    pub fn cube_id(state: &EditorState) -> NodeId {
        state.world.with_world(|scene| {
            scene
                .nodes()
                .iter()
                .find(|node| matches!(node.kind, NodeKind::Cube))
                .map(|node| node.id)
                .unwrap()
        })
    }
}
