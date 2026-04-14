use crate::{
    AssetCommand, DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
    EditorUiRouter, InspectorFieldChange, SelectionCommand, ViewportCommand,
};
use zircon_ui::UiBindingValue;

#[derive(Clone, Debug, PartialEq, Eq)]
enum MockEditorCommand {
    AddAnimationFrame { track_path: String, frame: u32 },
}

#[test]
fn animation_clip_binding_formats_as_stable_native_binding() {
    let binding = EditorUiBinding::new(
        "AnimationClipEditorView",
        "AddFrameButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::position_of_track_and_frame("root/child:transform", 24),
    );

    assert_eq!(
        binding.native_binding(),
        r#"AnimationClipEditorView/AddFrameButton:onClick(PositionOfTrackAndFrame("root/child:transform",24))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn editor_ui_router_dispatches_animation_binding_headlessly() {
    let binding = EditorUiBinding::new(
        "AnimationClipEditorView",
        "AddFrameButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::position_of_track_and_frame("root/child:transform", 24),
    );
    let mut router = EditorUiRouter::<MockEditorCommand>::default();
    router.register_exact(binding.path().clone(), |binding| match binding.payload() {
        EditorUiBindingPayload::PositionOfTrackAndFrame { track_path, frame } => {
            MockEditorCommand::AddAnimationFrame {
                track_path: track_path.clone(),
                frame: *frame,
            }
        }
        payload => panic!("unexpected payload {payload:?}"),
    });

    assert_eq!(
        router.dispatch(&binding),
        vec![MockEditorCommand::AddAnimationFrame {
            track_path: "root/child:transform".to_string(),
            frame: 24,
        }]
    );
}

#[test]
fn inspector_batch_binding_roundtrips_with_array_payload() {
    let binding = EditorUiBinding::new(
        "InspectorView",
        "ApplyBatchButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            vec![
                InspectorFieldChange::new("name", UiBindingValue::string("Batch Cube")),
                InspectorFieldChange::new("transform.translation.x", UiBindingValue::Float(4.0)),
                InspectorFieldChange::new("transform.translation.y", UiBindingValue::Float(5.0)),
            ],
        ),
    );

    assert_eq!(
        binding.native_binding(),
        r#"InspectorView/ApplyBatchButton:onClick(InspectorFieldBatch("entity://selected",[["name","Batch Cube"],["transform.translation.x",4.0],["transform.translation.y",5.0]]))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn dock_command_binding_roundtrips_through_native_binding() {
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
        binding.native_binding(),
        r#"HierarchyView/AutoHideDrawer:onClick(DockCommand.SetDrawerMode("left_top","AutoHide"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn activity_rail_toggle_binding_roundtrips_through_native_binding() {
    let binding = EditorUiBinding::new(
        "ActivityRail",
        "ProjectToggle",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::ActivateDrawerTab {
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"ActivityRail/ProjectToggle:onClick(DockCommand.ActivateDrawerTab("left_top","editor.project#1"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn dock_preset_command_bindings_roundtrip_through_native_binding() {
    let save_binding = EditorUiBinding::new(
        "ToolWindow",
        "SavePreset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::dock_command(DockCommand::SavePreset {
            name: "rider".to_string(),
        }),
    );

    assert_eq!(
        save_binding.native_binding(),
        r#"ToolWindow/SavePreset:onClick(DockCommand.SavePreset("rider"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&save_binding.native_binding()).unwrap(),
        save_binding
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
        load_binding.native_binding(),
        r#"ToolWindow/LoadPreset:onClick(DockCommand.LoadPreset("rider"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&load_binding.native_binding()).unwrap(),
        load_binding
    );
}

#[test]
fn dock_attach_command_bindings_roundtrip_through_native_binding() {
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
        drawer_binding.native_binding(),
        r#"ToolWindow/DropToRight:onClick(DockCommand.AttachViewToDrawer("editor.project#1","right_top"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&drawer_binding.native_binding()).unwrap(),
        drawer_binding
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
        document_binding.native_binding(),
        r#"DocumentTabs/DropToDocument:onClick(DockCommand.AttachViewToDocument("editor.project#1","workbench"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&document_binding.native_binding()).unwrap(),
        document_binding
    );
}

#[test]
fn viewport_command_binding_roundtrips_with_resize_event_kind() {
    let binding = EditorUiBinding::new(
        "SceneView",
        "ViewportSurface",
        EditorUiEventKind::Resize,
        EditorUiBindingPayload::viewport_command(ViewportCommand::Resized {
            width: 1024,
            height: 768,
        }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"SceneView/ViewportSurface:onResize(ViewportCommand.Resized(1024,768))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn selection_command_binding_roundtrips_for_scene_node_selection() {
    let binding = EditorUiBinding::new(
        "HierarchyView",
        "SceneNodeSelect",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode { node_id: 3 }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"HierarchyView/SceneNodeSelect:onClick(SelectionCommand.SelectSceneNode(3))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn asset_command_binding_roundtrips_for_asset_open() {
    let binding = EditorUiBinding::new(
        "ProjectView",
        "OpenAsset",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::asset_command(AssetCommand::OpenAsset {
            asset_path: "crate://prefabs/player.prefab".to_string(),
        }),
    );

    assert_eq!(
        binding.native_binding(),
        r#"ProjectView/OpenAsset:onClick(AssetCommand.OpenAsset("crate://prefabs/player.prefab"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}
