use super::*;
use crate::core::commands::{EditorCommandAction, EditorCommandDescriptor, EditorCommandRegistry};

#[test]
fn editor_command_registry_exposes_builtin_menu_operations_by_path() {
    use crate::core::editor_operation::EditorOperationPath;

    let registry = EditorCommandRegistry::default_workbench();
    let reset_path = EditorOperationPath::parse("window.layout.reset").unwrap();
    let reset = registry
        .command(&reset_path)
        .expect("reset layout command should be registered");

    assert_eq!(reset.id().as_str(), "window.layout.reset");
    assert_eq!(reset.display_name(), "Reset Layout");
    assert_eq!(reset.menu_path(), Some("Window/Reset Layout"));
    assert!(reset.callable_from_remote());
    assert!(matches!(reset.action(), EditorCommandAction::Operation));
    assert!(registry.operation_factory(&reset_path).is_none());

    for (path, menu_path) in [
        ("file.project.open", "File/Open Project"),
        ("file.project.save", "File/Save Project"),
        ("window.layout.save", "Window/Save Layout"),
        ("runtime.play_mode.enter", "Play/Enter Play Mode"),
        ("runtime.play_mode.exit", "Play/Exit Play Mode"),
        ("window.debug_observatory.open", "Window/Debug Observatory"),
        ("window.prefab_editor.open", "Window/Prefab Editor"),
        ("window.material_editor.open", "Window/Material Editor"),
        (
            "window.ui_component_showcase.open",
            "Window/UI Component Showcase",
        ),
        ("window.material_demo.open", "Window/Material Demo"),
        (
            "window.material_component_lab.open",
            "Window/Material Component Lab",
        ),
        ("window.ui_asset_editor.open", "Window/UI Asset Editor"),
        ("window.animation_editor.open", "Window/Animation Editor"),
        ("window.asset_browser.open", "Window/Asset Browser"),
        ("window.diagnostics.open", "Window/Diagnostics"),
        ("scene.node.create_camera", "Selection/Create Camera"),
        (
            "scene.node.create_ambient_light",
            "Selection/Create Ambient Light",
        ),
        (
            "scene.node.create_directional_light",
            "Selection/Create Directional Light",
        ),
        (
            "scene.node.create_point_light",
            "Selection/Create Point Light",
        ),
        (
            "scene.node.create_rect_light",
            "Selection/Create Rect Light",
        ),
        (
            "scene.node.create_spot_light",
            "Selection/Create Spot Light",
        ),
        ("view.plugin_manager.open", "View/Plugin Manager"),
        ("view.build_export.open", "View/Desktop Export"),
        ("inspector.field.apply_batch", "Inspector/Apply Changes"),
    ] {
        let descriptor = registry
            .command(&EditorOperationPath::parse(path).unwrap())
            .unwrap_or_else(|| panic!("{path} command should be registered"));
        assert_eq!(descriptor.menu_path(), Some(menu_path));
    }
}

#[test]
fn editor_operation_path_requires_namespace_action_and_leaf_segments() {
    use crate::core::editor_operation::EditorOperationPath;

    assert!(EditorOperationPath::parse("weather.cloud_layer.refresh").is_ok());
    assert!(EditorOperationPath::parse("view.weather.cloud_layers.open").is_ok());
    assert!(EditorOperationPath::parse("weather.refresh").is_err());
    assert!(EditorOperationPath::parse("Weather.CloudLayer.Refresh").is_err());
    assert!(EditorOperationPath::parse("weather.cloud_layer.re-fresh").is_err());
    assert!(EditorOperationPath::parse("weather.cloud layer.refresh").is_err());
}

#[test]
fn editor_operation_path_validation_streams_segments_without_collecting() {
    let source = include_str!("../../../core/editor_operation.rs");

    assert!(source.contains("let mut segment_count = 0;"));
    assert!(!source.contains("split('.').collect::<Vec<_>>()"));
}

#[test]
fn editor_operation_path_serde_enforces_canonical_parse() {
    use crate::core::editor_operation::EditorOperationPath;

    let path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let encoded = serde_json::to_value(&path).expect("serialize canonical operation path");
    assert_eq!(encoded, json!("weather.cloud_layer.refresh"));
    assert_eq!(
        serde_json::from_value::<EditorOperationPath>(encoded)
            .expect("deserialize canonical operation path"),
        path
    );

    for invalid in [
        "weather.refresh",
        "Weather.CloudLayer.Refresh",
        "weather.cloud_layer.re-fresh",
        "weather.cloud layer.refresh",
        "weather..refresh",
    ] {
        assert!(
            serde_json::from_value::<EditorOperationPath>(json!(invalid)).is_err(),
            "serde must reject non-canonical operation path `{invalid}`"
        );
    }
}

#[test]
fn editor_command_registry_rejects_invalid_menu_paths() {
    use crate::core::editor_operation::EditorOperationPath;

    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    for menu_path in [
        "",
        "Tools",
        "Tools//Refresh",
        "/Tools/Refresh",
        "Tools/Refresh/",
        "Tools/ Refresh",
    ] {
        let mut registry = EditorCommandRegistry::default();
        let error = registry
            .register(
                EditorCommandDescriptor::operation(operation_path.clone(), "Refresh Cloud Layers")
                    .with_menu_path(menu_path),
            )
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("editor command menu path `{menu_path}` is invalid")
        );
    }
}

#[test]
fn editor_extension_registry_collects_plugin_windows_menus_drawers_and_operations() {
    use crate::core::editor_extension::{
        ComponentDrawerDescriptor, DrawerDescriptor, EditorExtensionRegistry,
        EditorMenuItemDescriptor, EditorUiTemplateDescriptor, ViewDescriptor,
    };
    use crate::core::editor_operation::EditorOperationPath;

    let operation_path = EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap();
    let operation =
        EditorCommandDescriptor::operation(operation_path.clone(), "Refresh Cloud Layers")
            .with_menu_path("Tools/Weather/Refresh Cloud Layers");

    let mut registry = EditorExtensionRegistry::default();
    registry
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    registry
        .register_drawer(DrawerDescriptor::new(
            "weather.cloud_layers.drawer",
            "Cloud Layer Tools",
        ))
        .unwrap();
    registry
        .register_menu_item(
            EditorMenuItemDescriptor::new(
                "Tools/Weather/Refresh Cloud Layers",
                operation_path.clone(),
            )
            .with_priority(25)
            .with_shortcut("Ctrl+Alt+R")
            .with_enabled(false)
            .with_required_capabilities([
                "editor.extension.weather_authoring",
                "editor.extension.weather_authoring",
            ]),
        )
        .unwrap();
    registry
        .register_component_drawer(ComponentDrawerDescriptor::new(
            "weather.Component.CloudLayer",
            "asset://weather/editor/cloud_layer.inspector.zui",
            "weather.editor.CloudLayerInspectorController",
        ))
        .unwrap();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "weather.cloud_layer.inspector",
            "asset://weather/editor/cloud_layer.inspector.zui",
        ))
        .unwrap();
    registry.register_command(operation.clone()).unwrap();

    assert_eq!(registry.views().len(), 1);
    assert_eq!(registry.drawers().len(), 1);
    assert_eq!(registry.menu_items()[0].operation(), &operation_path);
    assert_eq!(registry.menu_items()[0].priority(), 25);
    assert_eq!(registry.menu_items()[0].shortcut(), Some("Ctrl+Alt+R"));
    assert!(!registry.menu_items()[0].enabled());
    assert_eq!(
        registry.menu_items()[0].required_capabilities(),
        &["editor.extension.weather_authoring".to_string()]
    );
    assert_eq!(
        registry.component_drawers()[0].component_type(),
        "weather.Component.CloudLayer"
    );
    assert_eq!(registry.pending_command(&operation_path), Some(&operation));

    let duplicate = registry.register_command(operation).unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("editor command weather.cloud_layer.refresh already registered"));
}

#[test]
fn operation_invocation_dispatches_to_the_same_event_and_marks_the_journal_record() {
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_invoke");
    let before = runtime.runtime.editor_snapshot().scene_entries.len();

    let record = runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();

    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateNode(NodeKind::Cube))
    );
    assert_eq!(
        record.operation_id.as_deref(),
        Some("scene.node.create_cube")
    );
    assert_eq!(
        record.operation_display_name.as_deref(),
        Some("Create Cube")
    );
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("scene.node.create_cube")
    );
    assert_eq!(
        runtime.runtime.editor_snapshot().scene_entries.len(),
        before + 1
    );
}

#[test]
fn editor_command_binding_invokes_the_shared_command_registry() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_command_operation_dispatch");
    let before = runtime.runtime.editor_snapshot().scene_entries.len();
    let binding = EditorUiBinding::new(
        "CommandPalette",
        "CreateCube",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::editor_command("scene.node.create_cube"),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("editor command binding should dispatch through the shared command registry");

    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateNode(NodeKind::Cube))
    );
    assert_eq!(
        record.operation_id.as_deref(),
        Some("scene.node.create_cube")
    );
    assert_eq!(
        runtime.runtime.editor_snapshot().scene_entries.len(),
        before + 1
    );
}

#[test]
fn operation_invocation_dispatches_rect_light_creation() {
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_rect_light");
    let before = runtime.runtime.editor_snapshot().scene_entries.len();

    let record = runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_rect_light").unwrap(),
            ),
        )
        .unwrap();

    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateNode(NodeKind::RectLight))
    );
    assert_eq!(
        record.operation_id.as_deref(),
        Some("scene.node.create_rect_light")
    );
    assert_eq!(
        record.operation_display_name.as_deref(),
        Some("Create Rect Light")
    );
    assert_eq!(
        runtime.runtime.editor_snapshot().scene_entries.len(),
        before + 1
    );
}

#[test]
fn operation_control_request_returns_structured_success_and_failure() {
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_control");

    let success = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            EditorOperationPath::parse("window.layout.reset").unwrap(),
        )),
    );
    assert!(success.error.is_none());
    assert_eq!(success.operation_id.as_deref(), Some("window.layout.reset"));
    assert!(success.value.is_some());

    let failure = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            EditorOperationPath::parse("weather.missing.action").unwrap(),
        )),
    );
    assert_eq!(
        failure.error.as_deref(),
        Some("editor command weather.missing.action is not registered")
    );
}

#[test]
fn failed_operation_control_request_is_journaled_without_creating_history() {
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_failure_journal");

    let failure = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            EditorOperationPath::parse("weather.missing.action").unwrap(),
        )),
    );

    assert_eq!(
        failure.error.as_deref(),
        Some("editor command weather.missing.action is not registered")
    );
    let journal = runtime.runtime.journal();
    assert_eq!(journal.records().len(), 1);
    let record = &journal.records()[0];
    assert_eq!(
        record.operation_id.as_deref(),
        Some("weather.missing.action")
    );
    assert_eq!(
        record.result.error.as_deref(),
        Some("editor command weather.missing.action is not registered")
    );

    let replay = EventRuntimeHarness::new("zircon_editor_event_operation_failure_journal_replay");
    EditorEventReplay::replay(&replay.runtime, journal.records()).expect("replay failure record");
    let replay_journal = replay.runtime.journal();
    assert_eq!(replay_journal.records().len(), 1);
    assert_eq!(
        replay_journal.records()[0].source,
        EditorEventSource::Replay
    );
    assert_eq!(
        replay_journal.records()[0].result.error.as_deref(),
        Some("editor command weather.missing.action is not registered")
    );
}

#[test]
fn failed_operation_control_request_preserves_operation_group_for_audit_delivery() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_failure_group");
    let listener_id = "External.OperationFailureAudit".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Operation Failure Audit".to_string(),
        },
    );

    let failure = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(
            EditorOperationInvocation::new(
                EditorOperationPath::parse("weather.missing.action").unwrap(),
            )
            .with_operation_group("External.Batch.42"),
        ),
    );

    assert_eq!(
        failure.error.as_deref(),
        Some("editor command weather.missing.action is not registered")
    );
    let journal = runtime.runtime.journal();
    assert_eq!(
        journal.records()[0].operation_group.as_deref(),
        Some("External.Batch.42")
    );

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    assert_eq!(
        deliveries.value["deliveries"][0]["operation_group"],
        json!("External.Batch.42")
    );
}

#[test]
fn remote_and_cli_operation_invocation_respects_callable_from_remote_gate() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_remote_gate");
    let operation_path = EditorOperationPath::parse("weather.secret.refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Refresh Secret Weather")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout))
                .with_callable_from_remote(false),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Secret Refresh",
            operation_path.clone(),
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime.runtime.refresh_reflection();

    let remote = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path.clone(),
        )),
    );
    assert_eq!(
        remote.error.as_deref(),
        Some("editor command weather.secret.refresh is not callable from remote control")
    );
    let cli = runtime
        .runtime
        .handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
                operation_path.clone(),
            )),
        );
    assert_eq!(
        cli.error.as_deref(),
        Some("editor command weather.secret.refresh is not callable from remote control")
    );
    assert_eq!(runtime.runtime.journal().records().len(), 2);
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("weather.secret.refresh")
    );
    assert_eq!(
        runtime.runtime.journal().records()[1].source,
        EditorEventSource::Cli
    );

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/tools/weather.secret.refresh"),
            action_id: "workbench.menu.item.click".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result) if result.error.is_none()
    ));
    assert_eq!(
        runtime.runtime.journal().records()[2]
            .operation_id
            .as_deref(),
        Some("weather.secret.refresh")
    );
}

#[test]
fn operation_control_request_lists_registered_operations_for_remote_discovery() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_listing");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");

    let response = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);

    assert!(response.error.is_none());
    let operations = response
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array");
    assert!(operations.iter().any(|operation| {
        operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some("window.layout.reset")
            && operation
                .get("menu_path")
                .and_then(serde_json::Value::as_str)
                == Some("Window/Reset Layout")
            && operation
                .get("undoable")
                .and_then(serde_json::Value::as_bool)
                == Some(false)
            && operation
                .get("required_capabilities")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|capabilities| capabilities.is_empty())
    }));
    assert!(operations.iter().any(|operation| {
        operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some("view.weather.cloud_layers.open")
            && operation
                .get("menu_path")
                .and_then(serde_json::Value::as_str)
                == Some("View/Weather/Cloud Layers")
            && operation
                .get("callable_from_remote")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
    }));

    let invocation = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            EditorOperationPath::parse("view.weather.cloud_layers.open").unwrap(),
        )),
    );
    assert!(
        invocation.error.is_none(),
        "a command listed by discovery must be invokable through the same registry: {:?}",
        invocation.error
    );
}

#[test]
fn operation_history_query_returns_global_transaction_snapshot() {
    use crate::core::editor_operation::EditorOperationControlRequest;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_history_pending");

    let response = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::QueryOperationHistory);

    assert!(response.error.is_none());
    let history = response.value.unwrap();
    assert_eq!(history["history"], "global");
    assert_eq!(history["len"], 0);
    assert_eq!(history["records"], serde_json::json!([]));
}

#[test]
fn operation_control_request_executes_registered_factory_through_transaction_engine() {
    use std::any::Any;
    use std::sync::Arc;

    use crate::core::commands::{EditorCommandCategory, EditorCommandDescriptor};
    use crate::core::editing::engine::{
        CommandExecutionError, EditCommand, EditContext, HistoryContextId,
    };
    use crate::core::editing::operation::{
        OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
        OperationCommandFactoryRegistration,
    };
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    };

    struct Factory;

    impl OperationCommandFactory for Factory {
        fn create(
            &self,
            _invocation: &EditorOperationInvocation,
        ) -> Result<OperationCommand, OperationCommandFactoryError> {
            Ok(OperationCommand::new(
                Box::new(Command),
                HistoryContextId::Global,
            ))
        }
    }

    struct Command;

    impl EditCommand for Command {
        fn label(&self) -> &str {
            "Execute Factory"
        }

        fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            Ok(())
        }

        fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_factory");
    let operation_id = EditorOperationPath::parse("test.operation.execute_factory").unwrap();
    runtime
        .runtime
        .commands()
        .lock()
        .register_operation(
            EditorCommandDescriptor::operation(operation_id.clone(), "Execute Factory"),
            OperationCommandFactoryRegistration::new(
                operation_id.clone(),
                "Execute Factory",
                Arc::new(Factory),
            ),
        )
        .unwrap();

    let invocation = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_id,
        )),
    );
    assert!(invocation.error.is_none(), "{:?}", invocation.error);

    let history = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::QueryOperationHistory)
        .value
        .unwrap();
    assert_eq!(history["len"], 1);
    assert_eq!(history["records"][0]["label"], "Execute Factory");
    assert_eq!(history["records"][0]["command_count"], 1);
}
