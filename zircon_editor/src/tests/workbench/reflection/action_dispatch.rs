use std::fs;
use std::path::Path;

use crate::core::asset::AssetToolkitOpenRoute;
use crate::core::editor_event::{EditorAssetEvent, EditorEvent, EditorEventSource};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::ViewDescriptorId;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime_interface::ui::{
    binding::UiBindingValue, event_ui::UiControlRequest, event_ui::UiControlResponse,
    event_ui::UiNodePath,
};

fn scalar_channel(value: f32) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Scalar(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn write_sequence_asset(path: &Path) {
    let asset = AnimationSequenceAsset {
        name: Some("Reflection Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: scalar_channel(1.0),
            }],
        }],
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

#[test]
fn workbench_reflection_call_action_dispatches_docking_inspector_and_viewport_actions() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_runtime");

    let inspector = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
            action_id: "inspector.apply_batch.invoke".to_string(),
            arguments: vec![
                UiBindingValue::string("entity://selected"),
                UiBindingValue::array(vec![
                    UiBindingValue::array(vec![
                        UiBindingValue::string("name"),
                        UiBindingValue::string("Bound Cube"),
                    ]),
                    UiBindingValue::array(vec![
                        UiBindingValue::string("transform.translation.x"),
                        UiBindingValue::Float(4.0),
                    ]),
                ]),
            ],
        });
    assert!(matches!(
        inspector,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    let editor_snapshot = runtime.runtime.editor_snapshot();
    assert_eq!(
        editor_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Bound Cube")
    );
    assert_eq!(
        editor_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.translation[0].as_str()),
        Some("4.00")
    );

    let viewport = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
            action_id: "workbench.viewport.resize".to_string(),
            arguments: vec![
                UiBindingValue::Unsigned(1024),
                UiBindingValue::Unsigned(768),
            ],
        });
    assert!(matches!(
        viewport,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(
        runtime.runtime.editor_snapshot().viewport_size,
        zircon_runtime_interface::math::UVec2::new(1024, 768)
    );

    let docking = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
            action_id: "workbench.view.detach_to_window".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        docking,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(runtime.runtime.current_layout().floating_windows.len(), 1);
}

#[test]
fn remote_control_operation_binding_preserves_native_binding_provenance_and_transaction() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_operation_provenance");
    let scene_entries_before = runtime.runtime.editor_snapshot().scene_entries.len();
    let binding = EditorUiBinding::new(
        "SceneMenu",
        "CreateCube",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::editor_operation("scene.node.create_cube"),
    );
    let binding_path = binding.path().native_prefix();

    let response = runtime
        .runtime
        .handle_control_request(UiControlRequest::InvokeBinding {
            binding: binding.as_ui_binding(),
        });
    assert!(matches!(
        response,
        UiControlResponse::Invocation(result) if result.error.is_none() && result.value.is_some()
    ));

    let journal = runtime.runtime.journal();
    let record = journal
        .records()
        .last()
        .expect("operation binding must append an event record");
    assert_eq!(record.source, EditorEventSource::Headless);
    assert_eq!(
        record.operation_id.as_deref(),
        Some("scene.node.create_cube")
    );
    assert_eq!(record.binding_path.as_deref(), Some(binding_path.as_str()));
    assert!(
        record.transaction_id.is_some(),
        "a mutating operation binding must retain its transaction provenance"
    );
    assert_eq!(record.save_generation, None);
    assert_eq!(
        runtime.runtime.editor_snapshot().scene_entries.len(),
        scene_entries_before + 1
    );
}

#[test]
fn remote_invoke_binding_and_route_cannot_bypass_command_surface_policy() {
    use crate::core::commands::EditorCommandDescriptor;
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;
    use crate::ui::workbench::event::MenuAction;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_remote_route_gate");
    let operation = EditorOperationPath::parse("weather.secret.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation.clone())
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout))
                .with_callable_from_remote(false),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::for_operation(operation.clone()))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .unwrap();
    runtime.runtime.refresh_reflection();

    let binding = EditorUiBinding::new(
        "RemoteProbe",
        "Invoke",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::editor_operation(operation.as_str()),
    );
    let direct = runtime
        .runtime
        .handle_control_request(UiControlRequest::InvokeBinding {
            binding: binding.as_ui_binding(),
        });
    assert!(matches!(
        direct,
        UiControlResponse::Invocation(result)
            if result.error.as_ref().is_some_and(|error| error.to_string().contains(
                "weather.secret.refresh is not callable from remote control"
            ))
    ));

    let menu_node = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/tools/weather.secret.refresh"),
        });
    let route_id = match menu_node {
        UiControlResponse::Node(Some(node)) => node.actions["workbench.menu.item.click"]
            .route_id
            .expect("registered menu action should expose a route"),
        response => panic!("expected reflected menu node, got {response:?}"),
    };
    let routed = runtime
        .runtime
        .handle_control_request(UiControlRequest::InvokeRoute {
            route_id,
            arguments: Vec::new(),
        });
    assert!(matches!(
        routed,
        UiControlResponse::Invocation(result)
            if result.error.as_ref().is_some_and(|error| error.to_string().contains(
                "weather.secret.refresh is not callable from remote control"
            ))
    ));

    let journal = runtime.runtime.journal();
    assert_eq!(journal.records().len(), 2);
    assert!(journal.records().iter().all(|record| {
        record.source == EditorEventSource::Headless
            && record.operation_id.as_deref() == Some("weather.secret.refresh")
            && matches!(
                &record.event,
                EditorEvent::Operation(
                    crate::core::editor_event::EditorOperationEvent::ControlFailure { .. }
                )
            )
    }));
}

#[test]
fn workbench_reflection_call_action_dispatches_typed_draft_actions() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_draft_runtime");

    let inspector = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
            action_id: "inspector.field.edit".to_string(),
            arguments: vec![
                UiBindingValue::string("entity://selected"),
                UiBindingValue::string("name"),
                UiBindingValue::string("Drafted Cube"),
            ],
        });
    assert!(matches!(
        inspector,
        UiControlResponse::Invocation(result)
            if result.error.is_none()
                && result.value.is_some()
                && result
                    .binding
                    .as_ref()
                    .map(|binding| binding.path.control_id.as_str())
                    == Some("NameField")
    ));
    assert_eq!(
        runtime
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Drafted Cube")
    );

    let mesh_import = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/left_top/editor.assets#1"),
            action_id: "workbench.asset.mesh_import.path.set".to_string(),
            arguments: vec![UiBindingValue::string("E:/Models/cube.glb")],
        });
    assert!(matches!(
        mesh_import,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(
        runtime.runtime.editor_snapshot().mesh_import_path,
        "E:/Models/cube.glb"
    );
}

#[test]
fn workbench_reflection_call_action_dispatches_asset_import_action() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_asset_import_runtime");

    let import_model = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/left_top/editor.assets#1"),
            action_id: "workbench.asset.model.import".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        import_model,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(
        runtime
            .runtime
            .journal()
            .records()
            .last()
            .map(|record| &record.event),
        Some(&EditorEvent::Asset(EditorAssetEvent::ImportModel))
    );
}

#[test]
fn workbench_reflection_call_action_dispatches_animation_track_creation_from_inspector() {
    let _guard = env_lock().lock().unwrap();
    let mut runtime =
        EventRuntimeHarness::new("zircon_workbench_reflection_animation_track_runtime");
    let asset_locator = "res://animation/reflection.sequence.zranim";
    let catalog = runtime.open_project_with_assets(
        "zircon_workbench_reflection_animation_track_project",
        |project| write_sequence_asset(&project.source_path(asset_locator)),
    );
    assert!(
        catalog
            .assets
            .iter()
            .any(|asset| asset.locator == asset_locator),
        "reflection fixture sequence should be indexed by the project catalog"
    );

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.to_string(),
            }),
        )
        .expect("open animation sequence asset");

    let response = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
            action_id: "animation.track.create".to_string(),
            arguments: vec![UiBindingValue::string("Root/Hero:AnimationPlayer.weight")],
        });
    assert!(matches!(
        response,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("animation sequence view should remain open");
    let route: AssetToolkitOpenRoute =
        serde_json::from_value(instance.serializable_payload.clone()).unwrap();
    assert_eq!(route.asset_locator().to_string(), asset_locator);
    assert_eq!(
        route.open_operation().as_str(),
        "timeline_sequence.authoring.open"
    );
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should be queryable after inspector action");

    assert!(pane
        .track_items
        .contains(&"Root/Hero:AnimationPlayer.weight".to_string()));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Created animation track Root/Hero:AnimationPlayer.weight"
    );
}
