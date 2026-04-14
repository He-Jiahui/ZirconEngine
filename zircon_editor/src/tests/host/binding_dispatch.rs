use zircon_editor_ui::{
    AssetCommand, DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
    InspectorFieldChange, SelectionCommand, ViewportCommand,
};
use zircon_math::UVec2;
use zircon_ui::UiBindingValue;

use crate::{
    apply_inspector_binding, apply_selection_binding, apply_viewport_binding,
    dispatch_animation_binding, dispatch_asset_binding, dispatch_docking_binding,
    dispatch_selection_binding, AnimationHostEvent, AssetHostEvent, LayoutCommand,
    SelectionHostEvent,
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
    state.world.with_world(|scene: &zircon_scene::Scene| {
        let node = scene.find_node(cube).unwrap();
        assert_eq!(node.name, "Bound Cube");
        assert_eq!(
            node.transform.translation,
            zircon_math::Vec3::new(4.0, 5.0, 6.0)
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
fn animation_binding_dispatches_into_host_event() {
    let binding = EditorUiBinding::new(
        "AnimationClipEditorView",
        "AddFrameButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::position_of_track_and_frame("root/child:transform", 24),
    );

    assert_eq!(
        dispatch_animation_binding(&binding).unwrap(),
        AnimationHostEvent::AddFrame {
            track_path: "root/child:transform".to_string(),
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
    assert_eq!(
        state.world.with_world(|scene| scene.selected_node()),
        Some(cube)
    );
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

mod support {
    use zircon_math::UVec2;
    use zircon_scene::{DefaultLevelManager, NodeId, NodeKind};

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
