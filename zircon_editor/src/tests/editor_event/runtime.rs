use crate::ui::{DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use serde_json::json;
use std::fs;
use zircon_runtime::core::resource::ResourceKind;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::ui::{
    event_ui::UiControlRequest, event_ui::UiControlResponse, event_ui::UiNodePath,
};

use crate::core::editor_event::{
    host_adapter, EditorAssetEvent, EditorEventReplay, EditorEventSource, EditorEventTransient,
};
use crate::{menu_action_binding, EditorEvent, LayoutCommand, MenuAction, WorkbenchLayout};

use super::support::{env_lock, EventRuntimeHarness};

#[test]
fn slint_adapter_binding_and_call_action_share_the_same_normalized_menu_event() {
    let _guard = env_lock().lock().unwrap();

    let slint = EventRuntimeHarness::new("zircon_editor_event_slint");
    let binding = EventRuntimeHarness::new("zircon_editor_event_binding");
    let action = EventRuntimeHarness::new("zircon_editor_event_action");

    let slint_before = slint.runtime.editor_snapshot().scene_entries.len();
    let binding_before = binding.runtime.editor_snapshot().scene_entries.len();
    let action_before = action.runtime.editor_snapshot().scene_entries.len();

    let slint_record = slint
        .runtime
        .dispatch_envelope(host_adapter::slint_menu_action("CreateNode.Cube").unwrap())
        .unwrap();
    let binding_record = binding
        .runtime
        .dispatch_binding(
            menu_action_binding(&MenuAction::CreateNode(NodeKind::Cube)),
            EditorEventSource::Headless,
        )
        .unwrap();

    let action_response = action
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/selection/CreateNode.Cube"),
            action_id: "onClick".to_string(),
            arguments: Vec::new(),
        });
    let UiControlResponse::Invocation(action_result) = action_response else {
        panic!("expected invocation response");
    };

    assert_eq!(
        slint_record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateNode(NodeKind::Cube))
    );
    assert_eq!(binding_record.event, slint_record.event);
    assert_eq!(
        action.runtime.journal().records()[0].event,
        slint_record.event
    );
    assert_eq!(binding_record.result.value, slint_record.result.value);
    assert_eq!(action_result.value, slint_record.result.value);

    assert_eq!(
        slint.runtime.editor_snapshot().scene_entries.len(),
        slint_before + 1
    );
    assert_eq!(
        binding.runtime.editor_snapshot().scene_entries.len(),
        binding_before + 1
    );
    assert_eq!(
        action.runtime.editor_snapshot().scene_entries.len(),
        action_before + 1
    );

    let serialized = serde_json::to_string(&slint_record).unwrap();
    assert!(
        !serialized.contains("WorkbenchMenuBar"),
        "canonical event record leaked slint view ids: {serialized}"
    );
}

#[test]
fn serialized_journal_replays_editor_and_layout_state_through_the_same_runtime_path() {
    let _guard = env_lock().lock().unwrap();

    let source = EventRuntimeHarness::new("zircon_editor_event_replay_source");
    source
        .runtime
        .dispatch_envelope(host_adapter::slint_menu_action("CreateNode.Cube").unwrap())
        .unwrap();
    source
        .runtime
        .dispatch_binding(
            EditorUiBinding::new(
                "ToolWindow",
                "AutoHideLeftTop",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::dock_command(DockCommand::SetDrawerMode {
                    slot: "left_top".to_string(),
                    mode: "AutoHide".to_string(),
                }),
            ),
            EditorEventSource::Headless,
        )
        .unwrap();

    let source_records = source.runtime.journal().records().to_vec();
    let serialized = serde_json::to_string(&source_records).unwrap();
    assert!(
        !serialized.contains("ToolWindow"),
        "journal should serialize semantic editor events instead of control ids: {serialized}"
    );

    let replay = EventRuntimeHarness::new("zircon_editor_event_replay_target");
    EditorEventReplay::replay(&replay.runtime, &source_records).unwrap();

    let source_snapshot = source.runtime.editor_snapshot();
    let replay_snapshot = replay.runtime.editor_snapshot();
    let source_layout: WorkbenchLayout = source.runtime.current_layout();
    let replay_layout: WorkbenchLayout = replay.runtime.current_layout();

    assert_eq!(
        source_snapshot.scene_entries.len(),
        replay_snapshot.scene_entries.len()
    );
    assert_eq!(
        source_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.clone()),
        replay_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.clone())
    );
    assert_eq!(source_layout, replay_layout);
    assert_eq!(
        replay.runtime.journal().records().len(),
        source_records.len()
    );
}

#[test]
fn transient_state_projects_into_reflection_without_reading_a_live_ui_tree() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_transient");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Slint,
            EditorEvent::Transient(EditorEventTransient::HoverNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
                hovered: true,
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Slint,
            EditorEvent::Transient(EditorEventTransient::FocusNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Slint,
            EditorEvent::Transient(EditorEventTransient::PressNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
                pressed: true,
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Slint,
            EditorEvent::Transient(EditorEventTransient::SetDrawerResizing {
                drawer_id: "left_top".to_string(),
                resizing: true,
            }),
        )
        .unwrap();

    let scene_node = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
        });
    let UiControlResponse::Node(Some(scene_node)) = scene_node else {
        panic!("expected scene node");
    };
    assert_eq!(
        scene_node.properties["transient.hovered"].reflected_value,
        json!(true)
    );
    assert_eq!(
        scene_node.properties["transient.focused"].reflected_value,
        json!(true)
    );
    assert!(scene_node.state_flags.pressed);

    let drawer_node = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/drawers/left_top"),
        });
    let UiControlResponse::Node(Some(drawer_node)) = drawer_node else {
        panic!("expected drawer node");
    };
    assert_eq!(
        drawer_node.properties["transient.resizing"].reflected_value,
        json!(true)
    );
}

#[test]
fn open_project_menu_event_requests_welcome_surface_without_project_open_side_effects() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_open_project");
    let record = runtime
        .runtime
        .dispatch_binding(
            menu_action_binding(&MenuAction::OpenProject),
            EditorEventSource::Headless,
        )
        .unwrap();

    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenProject)
    );
    assert!(record
        .effects
        .contains(&crate::EditorEventEffect::PresentWelcomeRequested));
    assert!(!record
        .effects
        .contains(&crate::EditorEventEffect::ProjectOpenRequested));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Open an existing project or create a renderable empty project."
    );
}

#[test]
fn slint_preset_menu_actions_normalize_to_layout_events_with_expected_names() {
    let save = host_adapter::slint_menu_action("SavePreset.rider").unwrap();
    let load = host_adapter::slint_menu_action("LoadPreset.").unwrap();

    assert_eq!(
        save.event,
        EditorEvent::Layout(LayoutCommand::SavePreset {
            name: "rider".to_string(),
        })
    );
    assert_eq!(
        load.event,
        EditorEvent::Layout(LayoutCommand::LoadPreset {
            name: "current".to_string(),
        })
    );
}

#[test]
fn scene_menu_actions_dispatch_through_runtime_and_only_update_status_line() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_scene_menu_actions");
    let open_record = runtime
        .runtime
        .dispatch_binding(
            menu_action_binding(&MenuAction::OpenScene),
            EditorEventSource::Headless,
        )
        .unwrap();
    let create_record = runtime
        .runtime
        .dispatch_binding(
            menu_action_binding(&MenuAction::CreateScene),
            EditorEventSource::Headless,
        )
        .unwrap();

    assert_eq!(
        open_record.event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenScene)
    );
    assert_eq!(
        create_record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateScene)
    );
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Scene open/create workflow is not wired yet"
    );
    assert!(!open_record
        .effects
        .contains(&crate::EditorEventEffect::LayoutChanged));
    assert!(!create_record
        .effects
        .contains(&crate::EditorEventEffect::LayoutChanged));
}

#[test]
fn close_view_layout_event_removes_the_view_instance_from_runtime_registry_state() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_close_view");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::WorkbenchMenu(MenuAction::OpenView(crate::ViewDescriptorId::new(
                "editor.asset_browser",
            ))),
        )
        .unwrap();

    let opened_instance = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == crate::ViewDescriptorId::new("editor.asset_browser")
        })
        .expect("asset browser view should open");

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Layout(crate::LayoutCommand::CloseView {
                instance_id: opened_instance.instance_id.clone(),
            }),
        )
        .unwrap();

    assert!(
        runtime
            .runtime
            .current_view_instances()
            .into_iter()
            .all(|instance| instance.instance_id != opened_instance.instance_id),
        "closed view instance should be removed from runtime session registry"
    );
}

#[test]
fn draft_inspector_binding_normalizes_and_updates_live_snapshot() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_draft_inspector");
    let binding = EditorUiBinding::parse_native_binding(
        r#"InspectorView/NameField:onChange(DraftCommand.SetInspectorField("entity://selected","name","Draft Cube"))"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("draft inspector binding should dispatch through runtime");

    assert_eq!(
        runtime
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Draft Cube")
    );
    assert!(record
        .effects
        .contains(&crate::EditorEventEffect::PresentationChanged));
    assert!(!record
        .effects
        .contains(&crate::EditorEventEffect::RenderChanged));
    assert!(!record
        .effects
        .contains(&crate::EditorEventEffect::LayoutChanged));
}

#[test]
fn draft_mesh_import_path_binding_normalizes_and_updates_live_snapshot() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_draft_mesh_import");
    let binding = EditorUiBinding::parse_native_binding(
        r#"AssetsView/MeshImportPathEdited:onChange(DraftCommand.SetMeshImportPath("E:/Models/cube.glb"))"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("mesh import path draft binding should dispatch through runtime");

    assert_eq!(
        runtime.runtime.editor_snapshot().mesh_import_path,
        "E:/Models/cube.glb"
    );
    assert!(record
        .effects
        .contains(&crate::EditorEventEffect::PresentationChanged));
    assert!(!record
        .effects
        .contains(&crate::EditorEventEffect::RenderChanged));
    assert!(!record
        .effects
        .contains(&crate::EditorEventEffect::LayoutChanged));
}

#[test]
fn asset_import_binding_normalizes_to_runtime_host_request() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_asset_import");
    let binding = EditorUiBinding::parse_native_binding(
        r#"AssetsView/ImportModel:onClick(AssetCommand.ImportModel())"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("asset import binding should dispatch through runtime");

    assert_eq!(
        record.event,
        EditorEvent::Asset(crate::EditorAssetEvent::ImportModel)
    );
    assert!(record
        .effects
        .contains(&crate::EditorEventEffect::ImportModelRequested));
    assert!(!record
        .effects
        .contains(&crate::EditorEventEffect::LayoutChanged));
    assert!(!record
        .effects
        .contains(&crate::EditorEventEffect::RenderChanged));
}

#[test]
fn asset_open_event_opens_ui_asset_editor_for_ui_toml_source() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_ui_asset_open");
    let ui_asset_path = std::env::temp_dir().join("zircon_editor_event_ui_asset_open.ui.toml");
    fs::write(
        &ui_asset_path,
        r#"
[asset]
kind = "layout"
id = "editor.tests.runtime_ui_asset"
version = 1
display_name = "Runtime UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
props = { text = "Runtime" }
"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(crate::EditorAssetEvent::OpenAsset {
                asset_path: ui_asset_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open ui asset event");

    assert_eq!(
        record.event,
        EditorEvent::Asset(crate::EditorAssetEvent::OpenAsset {
            asset_path: ui_asset_path.to_string_lossy().into_owned(),
        })
    );
    assert!(record
        .effects
        .contains(&crate::EditorEventEffect::LayoutChanged));
    assert!(runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .any(|instance| instance.descriptor_id == crate::ViewDescriptorId::new("editor.ui_asset")));

    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn workbench_menu_open_ui_asset_opens_ui_asset_editor_for_shared_asset() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_menu_open_ui_asset");
    let ui_asset_path = std::env::temp_dir().join("zircon_editor_event_menu_open_ui_asset.ui.toml");
    fs::write(
        &ui_asset_path,
        r#"
[asset]
kind = "layout"
id = "editor.tests.menu_ui_asset"
version = 1
display_name = "Menu UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
props = { text = "Menu" }
"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: ui_asset_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("menu open ui asset");

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_path: ui_asset_path.to_string_lossy().into_owned(),
        })
    );
    assert!(record
        .effects
        .contains(&crate::EditorEventEffect::LayoutChanged));
    assert!(runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .any(|instance| instance.descriptor_id == crate::ViewDescriptorId::new("editor.ui_asset")));

    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn asset_kind_filter_event_accepts_physics_and_animation_asset_kinds() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_asset_kind_filters");
    for (kind, expected) in [
        ("PhysicsMaterial", ResourceKind::PhysicsMaterial),
        ("AnimationSequence", ResourceKind::AnimationSequence),
        ("AnimationGraph", ResourceKind::AnimationGraph),
        ("AnimationStateMachine", ResourceKind::AnimationStateMachine),
    ] {
        let record = runtime
            .runtime
            .dispatch_event(
                EditorEventSource::Headless,
                EditorEvent::Asset(EditorAssetEvent::SetKindFilter {
                    kind: Some(kind.to_string()),
                }),
            )
            .expect("asset kind filter event");

        assert_eq!(
            runtime.runtime.editor_snapshot().asset_activity.kind_filter,
            Some(expected)
        );
        assert_eq!(
            runtime.runtime.editor_snapshot().asset_browser.kind_filter,
            Some(expected)
        );
        assert!(record
            .effects
            .contains(&crate::EditorEventEffect::PresentationChanged));
    }
}
