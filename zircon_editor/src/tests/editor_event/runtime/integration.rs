use super::*;
use crate::core::asset::{
    AssetToolkitDescriptor, AssetToolkitOpenRoute, AssetTypeContribution, AssetTypeId,
};
use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_event::SelectionHostEvent;
use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
use crate::core::editor_message::{EditorMessagePayload, EditorTopic, TOPIC_SCENE_INSPECTION};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::project::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetFolderRecord,
};
use crate::ui::workbench::project::EditorProjectDocument;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::scene::components::NodeKind;

#[test]
fn retained_adapter_binding_and_call_action_share_the_same_normalized_menu_event() {
    let _guard = env_lock().lock().unwrap();

    let retained_host = EventRuntimeHarness::new("zircon_editor_event_retained_host");
    let binding = EventRuntimeHarness::new("zircon_editor_event_binding");
    let action = EventRuntimeHarness::new("zircon_editor_event_action");

    let retained_before = retained_host.runtime.editor_snapshot().scene_entries.len();
    let binding_before = binding.runtime.editor_snapshot().scene_entries.len();
    let action_before = action.runtime.editor_snapshot().scene_entries.len();

    let retained_record = retained_host
        .runtime
        .dispatch_envelope(retained_menu_action("workbench.scene.node.create.cube").unwrap())
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
            node_path: UiNodePath::new("editor/workbench/menu/selection/scene.node.create_cube"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: Vec::new(),
        });
    let UiControlResponse::Invocation(action_result) = action_response else {
        panic!("expected invocation response");
    };
    assert_eq!(action_result.error, None);

    assert_eq!(
        retained_record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateNode(NodeKind::Cube))
    );
    assert_eq!(binding_record.event, retained_record.event);
    assert_eq!(
        binding_record.operation_id.as_deref(),
        Some("scene.node.create_cube")
    );
    assert_eq!(
        action.runtime.journal().records()[0].event,
        retained_record.event
    );
    assert_eq!(
        action.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("scene.node.create_cube")
    );
    assert_eq!(binding_record.result.value, retained_record.result.value);
    assert_eq!(action_result.value, retained_record.result.value);

    assert_eq!(
        retained_host.runtime.editor_snapshot().scene_entries.len(),
        retained_before + 1
    );
    assert_eq!(
        binding.runtime.editor_snapshot().scene_entries.len(),
        binding_before + 1
    );
    assert_eq!(
        action.runtime.editor_snapshot().scene_entries.len(),
        action_before + 1
    );

    let serialized = serde_json::to_string(&retained_record).unwrap();
    assert!(
        !serialized.contains("WorkbenchMenuBar"),
        "canonical event record leaked deleted generated UI view ids: {serialized}"
    );
}

#[test]
fn serialized_journal_replays_editor_and_layout_state_through_the_same_runtime_path() {
    let _guard = env_lock().lock().unwrap();

    let source = EventRuntimeHarness::new("zircon_editor_event_replay_source");
    source
        .runtime
        .dispatch_envelope(retained_menu_action("workbench.scene.node.create.cube").unwrap())
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
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::HoverNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
                hovered: true,
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::FocusNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Transient(EditorEventTransient::PressNode {
                node_path: "editor/workbench/pages/workbench/editor.scene#1".to_string(),
                pressed: true,
            }),
        )
        .unwrap();
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
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
        .contains(&EditorEventEffect::PresentWelcomeRequested));
    assert!(!record
        .effects
        .contains(&EditorEventEffect::ProjectOpenRequested));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Open an existing project or create a renderable empty project."
    );
}

#[test]
fn replacing_the_editor_world_publishes_an_inspection_resync() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_world_replacement_inspection");
    let topic = EditorTopic::parse(TOPIC_SCENE_INSPECTION).expect("valid scene inspection topic");
    let subscriber = runtime
        .runtime
        .context()
        .bus()
        .register_subscriber([topic])
        .expect("register scene inspection subscriber");
    let replacement = zircon_runtime::scene::create_default_level(&runtime.core.handle())
        .expect("replacement level should build");
    let replacement_generation = replacement.snapshot().world_generation();

    runtime
        .runtime
        .replace_world(replacement, "replacement-project")
        .expect("runtime should adopt the replacement level");

    let deliveries = runtime.runtime.context().bus().drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 1);
    let EditorMessagePayload::SceneInspection(message) = deliveries[0].message().payload() else {
        panic!("world replacement must publish a typed scene inspection message");
    };
    assert_eq!(message.previous_generation(), None);
    assert!(message.requires_resync());
    assert_eq!(message.generation(), replacement_generation);
}

#[test]
fn save_project_marks_the_transaction_history_only_after_persisting_the_world() {
    let _guard = env_lock().lock().unwrap();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon_editor_save_history_{unique}"));
    let location = root
        .parent()
        .expect("temporary project root should have a parent");
    ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: root
                .file_name()
                .expect("temporary project root should have a name")
                .to_string_lossy()
                .into_owned(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .expect("renderable template project should be created");

    {
        let runtime = EventRuntimeHarness::new("zircon_editor_event_save_history");
        let manager = runtime
            .core
            .resolve_manager::<crate::ui::host::EditorManager>(
                crate::ui::host::module::EDITOR_MANAGER_NAME,
            )
            .expect("editor manager should resolve");
        let document = manager.open_project(&root).expect("project should open");
        let level = manager
            .create_runtime_level(document.world)
            .expect("opened project scene should create a runtime level");
        runtime
            .runtime
            .replace_world(level, root.to_string_lossy())
            .expect("runtime should adopt the opened project level");

        let cube = {
            let shell = runtime.runtime.shell().lock();
            shell
                .state
                .world
                .try_with_world(|scene| {
                    scene
                        .nodes()
                        .iter()
                        .find(|node| node.kind == NodeKind::Cube)
                        .map(|node| node.id)
                })
                .flatten()
                .expect("renderable template should contain a cube")
        };
        runtime
            .runtime
            .dispatch_event(
                EditorEventSource::RetainedHost,
                EditorEvent::Selection(SelectionHostEvent::SelectSceneNode { node_id: cube }),
            )
            .expect("hierarchy selection should dispatch");
        runtime
            .runtime
            .dispatch_event(
                EditorEventSource::RetainedHost,
                EditorEvent::Inspector(EditorInspectorEvent {
                    subject_path: "entity://selected".to_string(),
                    changes: vec![InspectorFieldChange::new(
                        "transform.translation.x",
                        UiBindingValue::string("4.25"),
                    )],
                }),
            )
            .expect("inspector transaction should dispatch");
        assert!(runtime
            .runtime
            .context()
            .transactions()
            .is_dirty(HistoryContextId::Global)
            .expect("transaction dirty state should be queryable"));

        let save_binding = menu_action_binding(&MenuAction::SaveProject);
        let save_binding_path = save_binding.path().native_prefix();
        let save_record = runtime
            .runtime
            .dispatch_binding(save_binding, EditorEventSource::RetainedHost)
            .expect("save project menu binding should dispatch");
        assert_eq!(
            save_record.binding_path.as_deref(),
            Some(save_binding_path.as_str())
        );
        assert_eq!(
            save_record.operation_id.as_deref(),
            Some("file.project.save")
        );
        assert_eq!(save_record.transaction_id, None);
        assert!(save_record.save_generation.is_some());
        assert!(!runtime
            .runtime
            .context()
            .transactions()
            .is_dirty(HistoryContextId::Global)
            .expect("successful save should mark the current history clean"));
    }

    let mut reopened = ProjectManager::open(&root).expect("saved project should reopen");
    reopened
        .scan_and_import()
        .expect("reopened project assets should scan");
    let document = EditorProjectDocument::load_from_project(&reopened)
        .expect("reopened project document should load");
    let cube_x = document
        .world
        .nodes()
        .iter()
        .find(|node| node.kind == NodeKind::Cube)
        .expect("reopened template should retain the cube")
        .transform
        .translation
        .x;
    assert_eq!(cube_x, 4.25);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn command_palette_command_requests_open_effect() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_command_palette_open");
    let binding = EditorUiBinding::new(
        "CommandPalette",
        "OpenCommandPalette",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::editor_command("editor.command.palette"),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("command palette editor command should dispatch");

    assert_eq!(
        record.event,
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette)
    );
    assert_eq!(
        record.effects,
        vec![EditorEventEffect::CommandPaletteOpenRequested]
    );
}

#[test]
fn material_component_lab_binding_records_feedback_without_business_effects() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_material_lab_feedback");
    let binding = EditorUiBinding::new(
        "MaterialComponentLab",
        "MaterialLabChips",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::Custom(UiBindingCall::new("MaterialComponentLab")),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("Material Lab prototype binding should dispatch");

    assert!(record.effects.is_empty());
    assert_eq!(record.before_revision, record.after_revision);
    assert_eq!(
        record.operation_group.as_deref(),
        Some("MaterialComponentLab")
    );
    assert!(record.result.error.is_none());

    let effects = dispatch_builtin_template_binding(&runtime.runtime, "MaterialLab/Chips/Click")
        .expect("Material Lab builtin binding should exist")
        .expect("Material Lab builtin binding should dispatch");
    let dirty_domains = effects.dirty_domains();
    assert!(dirty_domains.contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!dirty_domains.requires_layout());
    assert!(!dirty_domains.requires_presentation());
}

#[test]
fn retained_preset_menu_actions_normalize_to_layout_events_with_expected_names() {
    let save = retained_menu_action("workbench.layout.preset.save.rider").unwrap();
    let load = retained_menu_action("workbench.layout.preset.load.").unwrap();
    let legacy_save = retained_menu_action("SavePreset.rider").unwrap();
    let legacy_load = retained_menu_action("LoadPreset.").unwrap();

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
    assert_eq!(legacy_save.event, save.event);
    assert_eq!(legacy_load.event, load.event);
}

#[test]
fn scene_menu_actions_dispatch_through_runtime_and_request_picker_presentation() {
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
    assert!(open_record
        .effects
        .contains(&EditorEventEffect::OpenScenePickerRequested));
    assert!(create_record
        .effects
        .contains(&EditorEventEffect::CreateScenePickerRequested));
    assert!(!open_record
        .effects
        .contains(&EditorEventEffect::LayoutChanged));
    assert!(!create_record
        .effects
        .contains(&EditorEventEffect::LayoutChanged));
}

#[test]
fn close_view_layout_event_removes_the_view_instance_from_runtime_registry_state() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_close_view");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::WorkbenchMenu(MenuAction::OpenView(EventViewDescriptorId::new(
                "editor.asset_browser",
            ))),
        )
        .unwrap();

    let opened_instance = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.asset_browser"))
        .expect("asset browser view should open");

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Layout(LayoutCommand::CloseView {
                instance_id: EventViewInstanceId::new(opened_instance.instance_id.0.clone()),
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
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(!record.effects.contains(&EditorEventEffect::RenderChanged));
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
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
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(!record.effects.contains(&EditorEventEffect::RenderChanged));
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
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
        EditorEvent::Asset(EditorAssetEvent::ImportModel)
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::ImportModelRequested));
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
    assert!(!record.effects.contains(&EditorEventEffect::RenderChanged));
}

#[test]
fn asset_open_event_opens_the_indexed_registry_toolkit() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_indexed_asset_open");
    let asset_locator = "res://ui/runtime_ui_asset.zui";
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let open_operation =
        EditorOperationPath::parse("view.editor.ui_asset.integration.open").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "editor.ui_asset.integration",
            "UI Asset Integration",
            "Assets",
        ))
        .unwrap();
    extension
        .register_asset_type_contribution(AssetTypeContribution::augment(asset_type).with_toolkit(
            AssetToolkitDescriptor::new("editor.ui_asset.integration", open_operation),
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .unwrap();
    runtime.runtime.sync_asset_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(
            EditorAssetCatalogSnapshotRecord {
                project_name: "Indexed Asset Open".to_string(),
                project_root: "E:/IndexedAssetOpen".to_string(),
                assets_root: "E:/IndexedAssetOpen/assets".to_string(),
                cache_root: "E:/IndexedAssetOpen/.zircon/cache".to_string(),
                default_scene_uri: String::new(),
                catalog_revision: 1,
                folders: vec![EditorAssetFolderRecord {
                    folder_id: "res://".to_string(),
                    parent_folder_id: None,
                    locator_prefix: "res://".to_string(),
                    display_name: "Assets".to_string(),
                    child_folder_ids: Vec::new(),
                    direct_asset_uuids: vec!["11111111-1111-1111-1111-111111111111".to_string()],
                    recursive_asset_count: 1,
                }],
                assets: vec![EditorAssetCatalogRecord {
                    uuid: "11111111-1111-1111-1111-111111111111".to_string(),
                    id: "22222222-2222-2222-2222-222222222222".to_string(),
                    locator: asset_locator.to_string(),
                    kind: ResourceKind::UiLayout,
                    display_name: "runtime_ui_asset.zui".to_string(),
                    file_name: "runtime_ui_asset.zui".to_string(),
                    extension: "zui".to_string(),
                    preview_state: PreviewState::Dirty,
                    meta_path: "E:/IndexedAssetOpen/assets/ui/runtime_ui_asset.zui.zmeta"
                        .to_string(),
                    preview_artifact_path: String::new(),
                    source_mtime_unix_ms: 0,
                    source_hash: String::new(),
                    dirty: false,
                    diagnostics: Vec::new(),
                    direct_reference_uuids: Vec::new(),
                }],
            },
            1,
        ),
    ));

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.to_string(),
            }),
        )
        .unwrap();

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_locator: asset_locator.to_string(),
        })
    );
    assert!(record.effects.contains(&EditorEventEffect::LayoutChanged));
    let toolkit_view = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset.integration")
        })
        .expect("indexed asset toolkit view should open");
    let route: AssetToolkitOpenRoute =
        serde_json::from_value(toolkit_view.serializable_payload).unwrap();
    assert_eq!(route.asset_locator().to_string(), asset_locator);
    assert_eq!(
        route.open_operation(),
        &EditorOperationPath::parse("view.editor.ui_asset.integration.open").unwrap()
    );
}

#[test]
fn asset_open_event_does_not_infer_a_toolkit_from_the_file_suffix() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_suffix_only_asset_open");
    let ui_asset_path = std::env::temp_dir().join("zircon_editor_event_suffix_only_asset_open.zui");
    fs::write(
        &ui_asset_path,
        r#"
[asset]
kind = "view"
id = "editor.tests.non_zui_runtime_ui_asset"
version = 1
display_name = "Non-ZUI Runtime UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
props = { text = "Non-ZUI" }
"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: ui_asset_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("suffix-only asset event should be rejected by the indexed registry boundary");

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_locator: ui_asset_path.to_string_lossy().into_owned(),
        })
    );
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
    assert!(!runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .any(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset")));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        format!(
            "Invalid asset locator {}: resource locator is missing scheme: {}",
            ui_asset_path.to_string_lossy(),
            ui_asset_path.to_string_lossy(),
        )
    );

    let _ = fs::remove_file(ui_asset_path);
}
