use crate::{
    AssetCommand, DockCommand, DraftCommand, EditorUiBinding, EditorUiBindingPayload,
    EditorUiEventKind, EditorUiRouter, InspectorFieldChange, SelectionCommand, ViewportCommand,
    WelcomeCommand,
};
use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use zircon_runtime::ui::binding::UiBindingValue;

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
        EditorUiBindingPayload::position_of_track_and_frame("root/child:transform.translation", 24),
    );

    assert_eq!(
        binding.native_binding(),
        r#"AnimationClipEditorView/AddFrameButton:onClick(PositionOfTrackAndFrame("root/child:transform.translation",24))"#
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
        EditorUiBindingPayload::position_of_track_and_frame("root/child:transform.translation", 24),
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
            track_path: "root/child:transform.translation".to_string(),
            frame: 24,
        }]
    );
}

#[test]
fn animation_binding_payload_uses_shared_framework_track_path() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let payload_source = std::fs::read_to_string(crate_root.join("src/ui/binding/core/payload.rs"))
        .unwrap_or_default();
    let payload_codec_source =
        std::fs::read_to_string(crate_root.join("src/ui/binding/core/payload_codec.rs"))
            .unwrap_or_default();
    let dispatch_source =
        std::fs::read_to_string(crate_root.join("src/ui/binding_dispatch/animation/dispatch.rs"))
            .unwrap_or_default();
    let event_source = std::fs::read_to_string(
        crate_root.join("src/ui/binding_dispatch/animation/animation_host_event.rs"),
    )
    .unwrap_or_default();

    for required in ["AnimationTrackPath", "zircon_runtime::core::framework::animation"] {
        assert!(
            payload_source.contains(required)
                || payload_codec_source.contains(required)
                || dispatch_source.contains(required)
                || event_source.contains(required),
            "editor animation binding should route through shared framework track path `{required}`"
        );
    }
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
fn welcome_command_binding_roundtrips_through_native_binding() {
    let edit_binding = EditorUiBinding::new(
        "WelcomeSurface",
        "ProjectNameEdited",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::SetProjectName {
            value: "Sandbox".to_string(),
        }),
    );

    assert_eq!(
        edit_binding.native_binding(),
        r#"WelcomeSurface/ProjectNameEdited:onChange(WelcomeCommand.SetProjectName("Sandbox"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&edit_binding.native_binding()).unwrap(),
        edit_binding
    );

    let open_recent_binding = EditorUiBinding::new(
        "WelcomeSurface",
        "OpenRecentProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::welcome_command(WelcomeCommand::OpenRecentProject {
            path: "E:/Projects/Sandbox".to_string(),
        }),
    );

    assert_eq!(
        open_recent_binding.native_binding(),
        r#"WelcomeSurface/OpenRecentProject:onClick(WelcomeCommand.OpenRecentProject("E:/Projects/Sandbox"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&open_recent_binding.native_binding()).unwrap(),
        open_recent_binding
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
fn viewport_toolbar_command_bindings_roundtrip_through_native_binding() {
    let bindings = [
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTool(
                    SceneViewportTool::Rotate,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetTool("Rotate"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTransformSpace(
                    TransformSpace::Global,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetTransformSpace("Global"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetProjectionMode(
                    ProjectionMode::Orthographic,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetProjectionMode("Orthographic"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::AlignView(
                    ViewOrientation::NegZ,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.AlignView("NegZ"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetDisplayMode(
                    DisplayMode::WireOverlay,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetDisplayMode("WireOverlay"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetGridMode(
                    GridMode::VisibleAndSnap,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetGridMode("VisibleAndSnap"))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetTranslateSnap(2.0)),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetTranslateSnap(2.0))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetRotateSnapDegrees(
                    30.0,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetRotateSnapDegrees(30.0))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetScaleSnap(0.25)),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetScaleSnap(0.25))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPreviewLighting(
                    false,
                )),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetPreviewLighting(false))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetPreviewSkybox(false)),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetPreviewSkybox(false))"#,
        ),
        (
            EditorUiBinding::new(
                "SceneView",
                "ViewportToolbar",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::viewport_command(ViewportCommand::SetGizmosEnabled(false)),
            ),
            r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetGizmosEnabled(false))"#,
        ),
    ];

    for (binding, expected) in bindings {
        assert_eq!(binding.native_binding(), expected);
        assert_eq!(
            EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
            binding
        );
    }
}

#[test]
fn viewport_toolbar_command_binding_roundtrips_with_typed_settings_payload() {
    let binding = EditorUiBinding::new(
        "SceneView",
        "ViewportToolbar",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::viewport_command(ViewportCommand::SetDisplayMode(
            DisplayMode::WireOverlay,
        )),
    );

    assert_eq!(
        binding.native_binding(),
        r#"SceneView/ViewportToolbar:onClick(ViewportCommand.SetDisplayMode("WireOverlay"))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn viewport_toolbar_command_roundtrips_for_projection_alignment_and_snaps() {
    let commands = [
        ViewportCommand::SetTool(SceneViewportTool::Scale),
        ViewportCommand::SetTransformSpace(TransformSpace::Global),
        ViewportCommand::SetProjectionMode(ProjectionMode::Orthographic),
        ViewportCommand::AlignView(ViewOrientation::PosY),
        ViewportCommand::SetGridMode(GridMode::VisibleAndSnap),
        ViewportCommand::SetTranslateSnap(2.5),
        ViewportCommand::SetRotateSnapDegrees(30.0),
        ViewportCommand::SetScaleSnap(0.25),
        ViewportCommand::SetPreviewLighting(false),
        ViewportCommand::SetPreviewSkybox(false),
        ViewportCommand::SetGizmosEnabled(false),
        ViewportCommand::FrameSelection,
    ];

    for command in commands {
        let binding = EditorUiBinding::new(
            "SceneView",
            "ViewportToolbar",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::viewport_command(command),
        );
        assert_eq!(
            EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
            binding
        );
    }
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

#[test]
fn asset_command_binding_roundtrips_for_import_model() {
    let binding = EditorUiBinding::new(
        "AssetsView",
        "ImportModel",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::asset_command(AssetCommand::ImportModel),
    );

    assert_eq!(
        binding.native_binding(),
        r#"AssetsView/ImportModel:onClick(AssetCommand.ImportModel())"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn draft_command_bindings_parse_into_typed_payloads_instead_of_custom_calls() {
    let inspector = EditorUiBinding::parse_native_binding(
        r#"InspectorView/NameField:onChange(DraftCommand.SetInspectorField("entity://selected","name","Draft Cube"))"#,
    )
    .unwrap();
    assert!(
        !matches!(inspector.payload(), EditorUiBindingPayload::Custom(_)),
        "inspector draft edit should not remain a custom payload"
    );

    let mesh_import = EditorUiBinding::parse_native_binding(
        r#"AssetsView/MeshImportPathEdited:onChange(DraftCommand.SetMeshImportPath("E:/Models/cube.glb"))"#,
    )
    .unwrap();
    assert!(
        !matches!(mesh_import.payload(), EditorUiBindingPayload::Custom(_)),
        "mesh import path edit should not remain a custom payload"
    );
}

#[test]
fn inspector_draft_binding_with_arguments_rewrites_control_id_from_field_id() {
    let binding = EditorUiBinding::new(
        "InspectorView",
        "NameField",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::draft_command(DraftCommand::SetInspectorField {
            subject_path: "entity://selected".to_string(),
            field_id: "name".to_string(),
            value: UiBindingValue::string(String::new()),
        }),
    );

    let rebound = binding
        .with_arguments(vec![
            UiBindingValue::string("entity://selected"),
            UiBindingValue::string("transform.translation.y"),
            UiBindingValue::string("12.5"),
        ])
        .unwrap();

    assert_eq!(rebound.path().view_id, "InspectorView");
    assert_eq!(rebound.path().control_id, "PositionYField");
    assert_eq!(
        rebound.native_binding(),
        r#"InspectorView/PositionYField:onChange(DraftCommand.SetInspectorField("entity://selected","transform.translation.y","12.5"))"#
    );
}
