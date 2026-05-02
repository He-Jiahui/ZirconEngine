use crate::ui::binding::{
    AnimationCommand, DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use serde_json::json;
use std::fs;
use zircon_runtime::core::framework::animation::AnimationTrackPath;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::{
    binding::UiBindingValue, event_ui::UiControlRequest, event_ui::UiControlResponse,
    event_ui::UiNodePath,
};

use crate::core::editor_event::{
    EditorAnimationEvent, EditorAssetEvent, EditorEvent, EditorEventEffect, EditorEventReplay,
    EditorEventSource, EditorEventTransient, LayoutCommand, MenuAction,
    ViewDescriptorId as EventViewDescriptorId, ViewInstanceId as EventViewInstanceId,
};
use crate::ui::slint_host::callback_dispatch::slint_menu_action;
use crate::ui::workbench::event::menu_action_binding;
use crate::ui::workbench::layout::WorkbenchLayout;
use crate::ui::workbench::view::ViewDescriptorId;

use super::support::{env_lock, EventRuntimeHarness};

#[test]
fn editor_operation_registry_exposes_builtin_menu_operations_by_path() {
    use crate::core::editor_operation::{EditorOperationPath, EditorOperationRegistry};

    let registry = EditorOperationRegistry::with_builtin_operations();
    let reset_path = EditorOperationPath::parse("Window.Layout.Reset").unwrap();
    let reset = registry
        .descriptor(&reset_path)
        .expect("reset layout operation should be registered");

    assert_eq!(reset.path().as_str(), "Window.Layout.Reset");
    assert_eq!(reset.display_name(), "Reset Layout");
    assert_eq!(reset.menu_path(), Some("Window/Reset Layout"));
    assert!(reset.callable_from_remote());
    assert!(reset.undoable().is_some());
}

#[test]
fn editor_operation_path_requires_namespace_action_and_leaf_segments() {
    use crate::core::editor_operation::EditorOperationPath;

    assert!(EditorOperationPath::parse("Weather.CloudLayer.Refresh").is_ok());
    assert!(EditorOperationPath::parse("View.weather.cloud_layers.Open").is_ok());
    assert!(EditorOperationPath::parse("Weather.Refresh").is_err());
    assert!(EditorOperationPath::parse("Weather..Refresh").is_err());
    assert!(EditorOperationPath::parse("Weather.Cloud Layer.Refresh").is_err());
}

#[test]
fn editor_operation_registry_rejects_invalid_menu_paths() {
    use crate::core::editor_operation::{
        EditorOperationDescriptor, EditorOperationPath, EditorOperationRegistry,
    };

    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    for menu_path in [
        "",
        "Tools",
        "Tools//Refresh",
        "/Tools/Refresh",
        "Tools/Refresh/",
        "Tools/ Refresh",
    ] {
        let mut registry = EditorOperationRegistry::default();
        let error = registry
            .register(
                EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
                    .with_menu_path(menu_path),
            )
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("editor operation menu path `{menu_path}` is invalid")
        );
    }
}

#[test]
fn editor_extension_registry_collects_plugin_windows_menus_drawers_and_operations() {
    use crate::core::editor_extension::{
        ComponentDrawerDescriptor, DrawerDescriptor, EditorExtensionRegistry,
        EditorMenuItemDescriptor, EditorUiTemplateDescriptor, ViewDescriptor,
    };
    use crate::core::editor_operation::{
        EditorOperationDescriptor, EditorOperationPath, UndoableEditorOperation,
    };

    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let operation = EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
        .with_menu_path("Tools/Weather/Refresh Cloud Layers")
        .with_undoable(UndoableEditorOperation::new("Refresh Cloud Layers"));

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
            "asset://weather/editor/cloud_layer.inspector.ui.toml",
            "weather.editor.CloudLayerInspectorController",
        ))
        .unwrap();
    registry
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "weather.cloud_layer.inspector",
            "asset://weather/editor/cloud_layer.inspector.ui.toml",
        ))
        .unwrap();
    registry.register_operation(operation.clone()).unwrap();

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
    assert_eq!(
        registry.operations().descriptor(&operation_path),
        Some(&operation)
    );

    let duplicate = registry.register_operation(operation).unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("editor operation Weather.CloudLayer.Refresh already registered"));
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
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();

    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::CreateNode(NodeKind::Cube))
    );
    assert_eq!(
        record.operation_id.as_deref(),
        Some("Scene.Node.CreateCube")
    );
    assert_eq!(
        record.operation_display_name.as_deref(),
        Some("Create Cube")
    );
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("Scene.Node.CreateCube")
    );
    assert_eq!(
        runtime.runtime.operation_stack().undo_stack()[0]
            .operation_id
            .as_str(),
        "Scene.Node.CreateCube"
    );
    assert_eq!(runtime.runtime.operation_stack().undo_stack().len(), 1);
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
            EditorOperationPath::parse("Window.Layout.Reset").unwrap(),
        )),
    );
    assert!(success.error.is_none());
    assert_eq!(success.operation_id.as_deref(), Some("Window.Layout.Reset"));
    assert!(success.value.is_some());

    let failure = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            EditorOperationPath::parse("Weather.Missing.Action").unwrap(),
        )),
    );
    assert_eq!(
        failure.error.as_deref(),
        Some("editor operation Weather.Missing.Action is not registered")
    );
}

#[test]
fn failed_operation_control_request_is_journaled_without_polluting_undo_stack() {
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_failure_journal");

    let failure = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            EditorOperationPath::parse("Weather.Missing.Action").unwrap(),
        )),
    );

    assert_eq!(
        failure.error.as_deref(),
        Some("editor operation Weather.Missing.Action is not registered")
    );
    let journal = runtime.runtime.journal();
    assert_eq!(journal.records().len(), 1);
    let record = &journal.records()[0];
    assert_eq!(
        record.operation_id.as_deref(),
        Some("Weather.Missing.Action")
    );
    assert_eq!(
        record.result.error.as_deref(),
        Some("editor operation Weather.Missing.Action is not registered")
    );
    assert!(runtime.runtime.operation_stack().undo_stack().is_empty());
    assert!(runtime.runtime.operation_stack().redo_stack().is_empty());

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
        Some("editor operation Weather.Missing.Action is not registered")
    );
    assert!(replay.runtime.operation_stack().undo_stack().is_empty());
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
                EditorOperationPath::parse("Weather.Missing.Action").unwrap(),
            )
            .with_operation_group("External.Batch.42"),
        ),
    );

    assert_eq!(
        failure.error.as_deref(),
        Some("editor operation Weather.Missing.Action is not registered")
    );
    let journal = runtime.runtime.journal();
    assert_eq!(
        journal.records()[0].operation_group.as_deref(),
        Some("External.Batch.42")
    );
    assert!(runtime.runtime.operation_stack().undo_stack().is_empty());

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
        EditorOperationControlRequest, EditorOperationDescriptor, EditorOperationInvocation,
        EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_remote_gate");
    let operation_path = EditorOperationPath::parse("Weather.Secret.Refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Refresh Secret Weather")
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
        Some("editor operation Weather.Secret.Refresh is not callable from remote control")
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
        Some("editor operation Weather.Secret.Refresh is not callable from remote control")
    );
    assert_eq!(runtime.runtime.journal().records().len(), 2);
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("Weather.Secret.Refresh")
    );
    assert_eq!(
        runtime.runtime.journal().records()[1].source,
        EditorEventSource::Cli
    );
    assert!(runtime.runtime.operation_stack().undo_stack().is_empty());

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/tools/Weather.Secret.Refresh"),
            action_id: "onClick".to_string(),
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
        Some("Weather.Secret.Refresh")
    );
}

#[test]
fn operation_control_request_lists_registered_operations_for_remote_discovery() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
    use crate::core::editor_operation::EditorOperationControlRequest;

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
            == Some("Window.Layout.Reset")
            && operation
                .get("menu_path")
                .and_then(serde_json::Value::as_str)
                == Some("Window/Reset Layout")
            && operation
                .get("undoable")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            && operation
                .get("required_capabilities")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|capabilities| capabilities.is_empty())
    }));
    assert!(operations.iter().any(|operation| {
        operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some("View.weather.cloud_layers.Open")
            && operation
                .get("menu_path")
                .and_then(serde_json::Value::as_str)
                == Some("View/Weather/Cloud Layers")
            && operation
                .get("callable_from_remote")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
    }));
}

#[test]
fn operation_control_request_returns_named_operation_history_stack() {
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_stack");
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();

    let response = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::QueryOperationStack);

    assert!(response.error.is_none());
    let value = response.value.as_ref().expect("stack value");
    let undo = value
        .get("undo_stack")
        .and_then(serde_json::Value::as_array)
        .expect("undo stack");
    assert_eq!(undo.len(), 1);
    assert_eq!(
        undo[0]
            .get("operation_id")
            .and_then(serde_json::Value::as_str),
        Some("Scene.Node.CreateCube")
    );
    assert_eq!(
        undo[0]
            .get("display_name")
            .and_then(serde_json::Value::as_str),
        Some("Create Cube")
    );
    assert_eq!(
        undo[0].get("source").and_then(serde_json::Value::as_str),
        Some("Headless")
    );
    assert_eq!(
        value
            .get("redo_stack")
            .and_then(serde_json::Value::as_array)
            .expect("redo stack")
            .len(),
        0
    );
}

#[test]
fn operation_stack_moves_entries_across_undo_and_redo_operations() {
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_stack_undo_redo");

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .expect("create cube operation");
    assert_eq!(runtime.runtime.operation_stack().undo_stack().len(), 1);
    assert!(runtime.runtime.operation_stack().redo_stack().is_empty());

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Edit.History.Undo").unwrap(),
            ),
        )
        .expect("undo operation");
    let stack_after_undo = runtime.runtime.operation_stack();
    assert!(
        stack_after_undo.undo_stack().is_empty(),
        "undo command should consume the previous undoable operation instead of adding itself"
    );
    assert_eq!(stack_after_undo.redo_stack().len(), 1);
    assert_eq!(
        stack_after_undo.redo_stack()[0].operation_id.as_str(),
        "Scene.Node.CreateCube"
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Edit.History.Redo").unwrap(),
            ),
        )
        .expect("redo operation");
    let stack_after_redo = runtime.runtime.operation_stack();
    assert_eq!(stack_after_redo.undo_stack().len(), 1);
    assert!(stack_after_redo.redo_stack().is_empty());
    assert_eq!(
        stack_after_redo.undo_stack()[0].operation_id.as_str(),
        "Scene.Node.CreateCube"
    );

    let response = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::QueryOperationStack);
    let value = response.value.as_ref().expect("stack value");
    assert_eq!(
        value["undo_stack"][0]["operation_id"].as_str(),
        Some("Scene.Node.CreateCube")
    );
    assert_eq!(value["redo_stack"].as_array().expect("redo stack").len(), 0);
}

#[test]
fn operation_stack_merges_continuous_invocations_with_same_operation_group() {
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_group_stack");
    let operation_path = EditorOperationPath::parse("Scene.Node.CreateCube").unwrap();

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation_path.clone())
                .with_operation_group("Viewport.TransformDrag.42"),
        )
        .expect("first grouped operation");
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation_path)
                .with_operation_group("Viewport.TransformDrag.42"),
        )
        .expect("second grouped operation");

    assert_eq!(
        runtime.runtime.journal().records().len(),
        2,
        "each grouped dispatch remains independently journaled"
    );
    let stack = runtime.runtime.operation_stack();
    assert_eq!(
        stack.undo_stack().len(),
        1,
        "continuous operations in the same group collapse to one history entry"
    );
    assert_eq!(
        stack.undo_stack()[0].operation_id.as_str(),
        "Scene.Node.CreateCube"
    );
    assert_eq!(
        stack.undo_stack()[0].operation_group.as_deref(),
        Some("Viewport.TransformDrag.42")
    );
    assert_eq!(
        stack.undo_stack()[0].sequence,
        2,
        "merged stack entry points at the latest grouped dispatch"
    );

    let response = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::QueryOperationStack);
    let value = response.value.as_ref().expect("stack value");
    assert_eq!(
        value["undo_stack"][0]["operation_group"].as_str(),
        Some("Viewport.TransformDrag.42")
    );
}

#[test]
fn operation_stack_preserves_original_source_across_undo_and_redo() {
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_stack_source");

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Cli,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .expect("cli create cube operation");
    assert_eq!(
        runtime.runtime.operation_stack().undo_stack()[0].source,
        EditorEventSource::Cli
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Edit.History.Undo").unwrap(),
            ),
        )
        .expect("undo operation");
    assert_eq!(
        runtime.runtime.operation_stack().redo_stack()[0].source,
        EditorEventSource::Cli
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Edit.History.Redo").unwrap(),
            ),
        )
        .expect("redo operation");
    assert_eq!(
        runtime.runtime.operation_stack().undo_stack()[0].source,
        EditorEventSource::Cli
    );
}

#[test]
fn operation_control_request_can_record_cli_source() {
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_cli_source");

    let response = runtime
        .runtime
        .handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
                EditorOperationPath::parse("Window.Layout.Reset").unwrap(),
            )),
        );

    assert!(response.error.is_none());
    assert_eq!(
        runtime.runtime.journal().records()[0].source,
        EditorEventSource::Cli
    );
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("Window.Layout.Reset")
    );
}

#[test]
fn event_listener_control_gates_named_event_deliveries() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_control");
    let listener_id = "External.HistoryPanel".to_string();

    let registered = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "History Panel".to_string(),
        },
    );
    assert!(registered.error.is_none());

    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetEnabled {
            listener_id: listener_id.clone(),
            enabled: false,
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();
    let disabled_deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries {
            listener_id: listener_id.clone(),
        },
    );
    assert_eq!(
        disabled_deliveries.value["deliveries"]
            .as_array()
            .expect("deliveries")
            .len(),
        0
    );

    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetEnabled {
            listener_id: listener_id.clone(),
            enabled: true,
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    assert_eq!(
        deliveries.value["deliveries"][0]["operation_id"],
        "Scene.Node.CreateCube"
    );
    assert_eq!(deliveries.value["deliveries"][0]["sequence"], 2);
}

#[test]
fn event_listener_filter_limits_delivery_by_operation_path_prefix() {
    use crate::core::editor_event::{EditorEventListenerControlRequest, EditorEventListenerFilter};
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_filter");
    let listener_id = "External.SceneHistory".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Scene History".to_string(),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetFilter {
            listener_id: listener_id.clone(),
            filter: EditorEventListenerFilter::operation_prefix("Scene.Node."),
        },
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Window.Layout.Reset").unwrap(),
            ),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["operation_id"], "Scene.Node.CreateCube");
}

#[test]
fn event_listener_filter_limits_delivery_by_operation_group() {
    use crate::core::editor_event::{EditorEventListenerControlRequest, EditorEventListenerFilter};
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_group_filter");
    let listener_id = "External.TransformDrag".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Transform Drag".to_string(),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetFilter {
            listener_id: listener_id.clone(),
            filter: EditorEventListenerFilter::operation_group("Viewport.TransformDrag.42"),
        },
    );

    let operation_path = EditorOperationPath::parse("Scene.Node.CreateCube").unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation_path.clone())
                .with_operation_group("Viewport.TransformDrag.41"),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation_path.clone())
                .with_operation_group("Viewport.TransformDrag.42"),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(
        deliveries[0]["operation_group"],
        json!("Viewport.TransformDrag.42")
    );
}

#[test]
fn event_listener_filter_limits_delivery_by_source_and_failure_state() {
    use crate::core::editor_event::{
        EditorEventListenerControlRequest, EditorEventListenerFilter, EditorEventSource,
    };
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_source_filter");
    let listener_id = "External.CliFailures".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "CLI Failure Monitor".to_string(),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetFilter {
            listener_id: listener_id.clone(),
            filter: EditorEventListenerFilter::source(EditorEventSource::Cli).failures_only(),
        },
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();
    let cli_success = runtime
        .runtime
        .handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            )),
        );
    assert!(cli_success.error.is_none());
    let cli_failure = runtime
        .runtime
        .handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
                EditorOperationPath::parse("Weather.Missing.Action").unwrap(),
            )),
        );
    assert!(cli_failure.error.is_some());

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["source"], "Cli");
    assert_eq!(deliveries[0]["operation_id"], "Weather.Missing.Action");
    let result_error = deliveries[0]["result"]["error"]
        .as_str()
        .expect("result error");
    assert!(result_error.contains("Weather.Missing.Action"));
    assert!(result_error.contains("is not registered"));
}

#[test]
fn event_listener_control_clears_operation_path_filter() {
    use crate::core::editor_event::{EditorEventListenerControlRequest, EditorEventListenerFilter};
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_clear_filter");
    let listener_id = "External.DynamicPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Dynamic Panel".to_string(),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetFilter {
            listener_id: listener_id.clone(),
            filter: EditorEventListenerFilter::operation_prefix("Scene.Node."),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::ClearFilter {
            listener_id: listener_id.clone(),
        },
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Window.Layout.Reset").unwrap(),
            ),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["operation_id"], "Window.Layout.Reset");
}

#[test]
fn event_listener_control_unregisters_listener_and_drops_deliveries() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_unregister");
    let listener_id = "External.TemporaryPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Temporary Panel".to_string(),
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();

    let unregistered = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Unregister {
            listener_id: listener_id.clone(),
        },
    );
    assert!(unregistered.error.is_none());

    let listeners = runtime
        .runtime
        .handle_event_listener_control_request(EditorEventListenerControlRequest::ListListeners);
    assert!(listeners.value["listeners"]
        .as_array()
        .expect("listeners")
        .is_empty());

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    assert_eq!(
        deliveries.error.as_deref(),
        Some("editor event listener External.TemporaryPanel is not registered")
    );
}

#[test]
fn event_listener_control_rejects_unknown_listener_queries() {
    use crate::core::editor_event::EditorEventListenerControlRequest;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_unknown_query");
    let listener_id = "External.MissingPanel".to_string();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries {
            listener_id: listener_id.clone(),
        },
    );
    assert_eq!(
        deliveries.error.as_deref(),
        Some("editor event listener External.MissingPanel is not registered")
    );

    let cursor = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesSince {
            listener_id: listener_id.clone(),
            after_sequence: 10,
        },
    );
    assert_eq!(
        cursor.error.as_deref(),
        Some("editor event listener External.MissingPanel is not registered")
    );

    let ack = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::AckDeliveriesThrough {
            listener_id,
            sequence: 10,
        },
    );
    assert_eq!(
        ack.error.as_deref(),
        Some("editor event listener External.MissingPanel is not registered")
    );
}

#[test]
fn event_listener_control_queries_deliveries_after_sequence_cursor() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_cursor");
    let listener_id = "External.PollingPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Polling Panel".to_string(),
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesSince {
            listener_id,
            after_sequence: 1,
        },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["sequence"], 2);
    assert_eq!(deliveries[0]["operation_id"], "Scene.Node.CreateCube");
}

#[test]
fn event_listener_control_acknowledges_deliveries_through_sequence() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_ack");
    let listener_id = "External.StreamingPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Streaming Panel".to_string(),
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();

    let ack = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::AckDeliveriesThrough {
            listener_id: listener_id.clone(),
            sequence: 1,
        },
    );
    assert!(ack.error.is_none());
    assert_eq!(ack.value["removed"], 1);

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["sequence"], 2);
}

#[test]
fn event_listener_control_reports_listener_status_with_pending_delivery_bounds() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_status");
    let listener_id = "External.StatusPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Status Panel".to_string(),
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("Scene.Node.CreateCube").unwrap(),
            ),
        )
        .unwrap();

    let status = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryListenerStatus {
            listener_id: listener_id.clone(),
        },
    );
    assert!(status.error.is_none());
    assert_eq!(status.value["listener_id"], listener_id);
    assert_eq!(status.value["descriptor"]["display_name"], "Status Panel");
    assert_eq!(status.value["descriptor"]["enabled"], true);
    assert_eq!(status.value["pending_delivery_count"], 2);
    assert_eq!(status.value["first_pending_sequence"], 1);
    assert_eq!(status.value["last_pending_sequence"], 2);

    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::AckDeliveriesThrough {
            listener_id: listener_id.clone(),
            sequence: 2,
        },
    );
    let empty_status = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryListenerStatus { listener_id },
    );
    assert_eq!(empty_status.value["pending_delivery_count"], 0);
    assert_eq!(empty_status.value["first_pending_sequence"], json!(null));
    assert_eq!(empty_status.value["last_pending_sequence"], json!(null));
}

#[test]
fn editor_runtime_accepts_plugin_extension_operations_for_later_invocation() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::{
        EditorOperationDescriptor, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_operation");
    let operation_path = EditorOperationPath::parse("Weather.Tools.ResetLayout").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Weather Reset Layout")
                .with_menu_path("Tools/Weather/Reset Layout")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    let record = runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

    assert_eq!(
        record.operation_id.as_deref(),
        Some("Weather.Tools.ResetLayout")
    );
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("Weather.Tools.ResetLayout")
    );
}

#[test]
fn explicit_plugin_operation_records_its_own_undo_stack_entry_when_reusing_builtin_event() {
    use crate::core::editor_extension::EditorExtensionRegistry;
    use crate::core::editor_operation::{
        EditorOperationDescriptor, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource, UndoableEditorOperation,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_operation_stack_identity");
    let operation_path = EditorOperationPath::parse("Zzz.Tools.ResetLayout").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Plugin Reset Layout")
                .with_menu_path("Tools/Zzz/Reset Layout")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout))
                .with_undoable(UndoableEditorOperation::new("Plugin Reset Layout")),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

    let stack = runtime.runtime.operation_stack();
    assert_eq!(stack.undo_stack().len(), 1);
    assert_eq!(
        stack.undo_stack()[0].operation_id.as_str(),
        "Zzz.Tools.ResetLayout"
    );
    assert_eq!(stack.undo_stack()[0].display_name, "Plugin Reset Layout");
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("Zzz.Tools.ResetLayout")
    );
}

#[test]
fn editor_runtime_projects_plugin_menu_operations_into_remote_callable_reflection() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_menu_operation");
    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(
            EditorMenuItemDescriptor::new("Tools/Weather/Refresh Cloud Layers", operation_path)
                .with_priority(10)
                .with_shortcut("Ctrl+Alt+R"),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime.runtime.refresh_reflection();

    let menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/tools/Weather.CloudLayer.Refresh"),
        });
    assert!(matches!(
        menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Refresh Cloud Layers"
                && node.actions["onClick"].binding_symbol == "EditorOperation"
                && node.actions["onClick"].callable_from_remote
                && node.properties["operation_path"].reflected_value
                    == json!("Weather.CloudLayer.Refresh")
                && node.properties["shortcut"].reflected_value == json!("Ctrl+Alt+R")
    ));

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/tools/Weather.CloudLayer.Refresh"),
            action_id: "onClick".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result)
            if result.error.is_none()
                && result.binding
                    .as_ref()
                    .and_then(|binding| binding.action.as_ref())
                    .map(|call| call.symbol.as_str())
                    == Some("EditorOperation")
    ));
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("Weather.CloudLayer.Refresh")
    );
}

#[test]
fn editor_operation_ui_binding_arguments_are_preserved_in_journal() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_menu_operation_arguments");
    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path,
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: "External.OperationAudit".to_string(),
            display_name: "Operation Audit".to_string(),
        },
    );
    runtime.runtime.refresh_reflection();

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/tools/Weather.CloudLayer.Refresh"),
            action_id: "onClick".to_string(),
            arguments: vec![
                UiBindingValue::String("storm".to_string()),
                UiBindingValue::Unsigned(7),
                UiBindingValue::Bool(true),
            ],
        });

    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result)
            if result.error.is_none()
                && result.binding
                    .as_ref()
                    .and_then(|binding| binding.action.as_ref())
                    .map(|call| call.arguments.len())
                    == Some(4)
    ));
    let journal = runtime.runtime.journal();
    let record = &journal.records()[0];
    assert_eq!(
        record.operation_id.as_deref(),
        Some("Weather.CloudLayer.Refresh")
    );
    assert_eq!(
        record.operation_arguments.as_ref(),
        Some(&json!(["storm", 7, true]))
    );
    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries {
            listener_id: "External.OperationAudit".to_string(),
        },
    );
    assert_eq!(
        deliveries.value["deliveries"][0]["operation_arguments"],
        json!(["storm", 7, true])
    );
}

#[test]
fn editor_runtime_registers_plugin_views_as_activity_descriptors() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_view_descriptor");
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
    runtime.runtime.refresh_reflection();

    let descriptor = runtime
        .runtime
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "weather.cloud_layers")
        .expect("plugin view descriptor registered");
    assert_eq!(descriptor.default_title, "Cloud Layers");
    assert_eq!(descriptor.icon_key, "weather.cloud_layers");
    assert!(runtime
        .runtime
        .activity_view_descriptor("weather.cloud_layers")
        .is_some());
}

#[test]
fn editor_runtime_projects_plugin_views_into_view_menu_operations() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_view_menu_operation");
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
    runtime.runtime.refresh_reflection();

    let menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/view/View.weather.cloud_layers.Open"),
        });
    assert!(matches!(
        menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Cloud Layers"
                && node.properties["operation_path"].reflected_value
                    == json!("View.weather.cloud_layers.Open")
                && node.actions["onClick"].binding_symbol == "EditorOperation"
                && node.actions["onClick"].callable_from_remote
    ));

    let invoked = runtime
        .runtime
        .handle_control_request(UiControlRequest::CallAction {
            node_path: UiNodePath::new("editor/workbench/menu/view/View.weather.cloud_layers.Open"),
            action_id: "onClick".to_string(),
            arguments: Vec::new(),
        });
    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result) if result.error.is_none()
    ));
    assert!(runtime
        .runtime
        .current_view_instances()
        .iter()
        .any(|instance| instance.descriptor_id.0 == "weather.cloud_layers"));
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("View.weather.cloud_layers.Open")
    );
}

#[test]
fn editor_runtime_consumes_plugin_registration_reports_with_capability_gate() {
    use crate::core::editor_extension::{
        EditorExtensionRegistry, EditorMenuItemDescriptor, ViewDescriptor,
    };
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationDescriptor, EditorOperationInvocation,
        EditorOperationPath,
    };
    use crate::core::editor_plugin::EditorPluginRegistrationReport;
    use crate::ui::host::module::EDITOR_MANAGER_NAME;
    use crate::ui::host::EditorManager;
    use zircon_runtime::{PluginModuleManifest, PluginPackageManifest};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::with_enabled_subsystems(
        "zircon_editor_event_plugin_registration_gate",
        &[],
    );
    let capability = "editor.extension.weather_authoring".to_string();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    extension
        .register_operation(
            EditorOperationDescriptor::new(operation_path.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path.clone(),
        ))
        .unwrap();

    runtime
        .runtime
        .register_editor_plugin_registration(EditorPluginRegistrationReport {
            package_manifest: PluginPackageManifest::new("weather", "Weather").with_editor_module(
                PluginModuleManifest::editor("weather.editor", "zircon_plugin_weather_editor")
                    .with_capabilities([capability.clone()]),
            ),
            capabilities: vec![capability.clone()],
            extensions: extension,
            diagnostics: Vec::new(),
        })
        .expect("register editor plugin report");
    runtime.runtime.refresh_reflection();

    assert!(runtime
        .runtime
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.descriptor_id.0 != "weather.cloud_layers"));
    let disabled_menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/view/View.weather.cloud_layers.Open"),
        });
    assert!(matches!(disabled_menu, UiControlResponse::Node(None)));
    let disabled_operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    assert!(!disabled_operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array")
        .iter()
        .any(|operation| operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some("Weather.CloudLayer.Refresh")));
    let disabled_invoke = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path.clone(),
        )),
    );
    assert_eq!(
        disabled_invoke.error.as_deref(),
        Some(
            "editor operation Weather.CloudLayer.Refresh requires disabled capabilities: editor.extension.weather_authoring"
        )
    );

    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    manager
        .set_editor_capabilities_enabled(&[capability], true)
        .unwrap();
    runtime.runtime.refresh_reflection();

    let descriptor = runtime
        .runtime
        .descriptors()
        .into_iter()
        .find(|descriptor| descriptor.descriptor_id.0 == "weather.cloud_layers")
        .expect("enabled plugin view descriptor registered");
    assert_eq!(
        descriptor.required_capabilities,
        vec!["editor.extension.weather_authoring"]
    );
    let enabled_menu = runtime
        .runtime
        .handle_control_request(UiControlRequest::QueryNode {
            node_path: UiNodePath::new("editor/workbench/menu/view/View.weather.cloud_layers.Open"),
        });
    assert!(matches!(
        enabled_menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Cloud Layers"
                && node.properties["operation_path"].reflected_value
                    == json!("View.weather.cloud_layers.Open")
    ));
    let enabled_operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    let enabled_operations = enabled_operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array");
    let weather_operation = enabled_operations
        .iter()
        .find(|operation| {
            operation
                .get("operation_id")
                .and_then(serde_json::Value::as_str)
                == Some("Weather.CloudLayer.Refresh")
        })
        .expect("weather operation is discoverable when capability is enabled");
    assert_eq!(
        weather_operation.get("required_capabilities"),
        Some(&json!(["editor.extension.weather_authoring"]))
    );
    assert!(enabled_operations.iter().any(|operation| operation
        .get("operation_id")
        .and_then(serde_json::Value::as_str)
        == Some("Weather.CloudLayer.Refresh")));
    let enabled_invoke = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path,
        )),
    );
    assert!(enabled_invoke.error.is_none());
}

#[test]
fn editor_runtime_exposes_plugin_component_drawer_templates_for_inspector_lookup() {
    use crate::core::editor_extension::{
        ComponentDrawerDescriptor, EditorExtensionRegistry, EditorUiTemplateDescriptor,
    };
    use crate::core::editor_operation::{EditorOperationDescriptor, EditorOperationPath};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_plugin_component_drawer");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_operation(EditorOperationDescriptor::new(
            EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap(),
            "Refresh Cloud Layers",
        ))
        .unwrap();
    extension
        .register_ui_template(EditorUiTemplateDescriptor::new(
            "weather.cloud_layer.inspector",
            "asset://weather/editor/cloud_layer.inspector.ui.toml",
        ))
        .unwrap();
    extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                "weather.Component.CloudLayer",
                "asset://weather/editor/cloud_layer.inspector.ui.toml",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_binding("Weather.CloudLayer.Refresh"),
        )
        .unwrap();

    runtime
        .runtime
        .register_editor_extension(extension)
        .expect("register editor extension");

    let drawer = runtime
        .runtime
        .component_drawer_descriptor("weather.Component.CloudLayer")
        .expect("component drawer registered");
    assert_eq!(
        drawer.ui_document(),
        "asset://weather/editor/cloud_layer.inspector.ui.toml"
    );
    assert_eq!(
        drawer.controller(),
        "weather.editor.CloudLayerInspectorController"
    );
    assert_eq!(drawer.bindings(), ["Weather.CloudLayer.Refresh"]);

    let template = runtime
        .runtime
        .ui_template_descriptor("weather.cloud_layer.inspector")
        .expect("ui template registered");
    assert_eq!(
        template.ui_document(),
        "asset://weather/editor/cloud_layer.inspector.ui.toml"
    );
}

#[test]
fn editor_runtime_rejects_menu_items_to_missing_operations() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_menu_missing_operation");
    let mut extension = EditorExtensionRegistry::default();
    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            operation_path,
        ))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor operation Weather.CloudLayer.Refresh is not registered"
    );
}

#[test]
fn editor_extension_registry_rejects_invalid_menu_item_paths() {
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::EditorOperationPath;

    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    for path in [
        "",
        "Tools",
        "Tools//Refresh",
        "/Tools/Refresh",
        "Tools/Refresh/",
    ] {
        let mut extension = EditorExtensionRegistry::default();
        let error = extension
            .register_menu_item(EditorMenuItemDescriptor::new(path, operation_path.clone()))
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("editor menu item path `{path}` is invalid")
        );
    }
}

#[test]
fn editor_extension_registry_rejects_view_ids_that_cannot_form_open_operation_paths() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};

    for view_id in ["weather/cloud_layers", "weather.cloud layers"] {
        let mut extension = EditorExtensionRegistry::default();
        let error = extension
            .register_view(ViewDescriptor::new(view_id, "Cloud Layers", "Weather"))
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("editor operation path `View.{view_id}.Open` is invalid")
        );
        assert!(extension.views().is_empty());
    }
}

#[test]
fn editor_runtime_rejects_duplicate_extension_view_without_registering_operations() {
    use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationDescriptor, EditorOperationPath,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_duplicate_extension_view");
    let mut first_extension = EditorExtensionRegistry::default();
    first_extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers",
            "Weather",
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(first_extension)
        .expect("register first extension view");

    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let mut duplicate_extension = EditorExtensionRegistry::default();
    duplicate_extension
        .register_view(ViewDescriptor::new(
            "weather.cloud_layers",
            "Cloud Layers Duplicate",
            "Weather",
        ))
        .unwrap();
    duplicate_extension
        .register_operation(EditorOperationDescriptor::new(
            operation_path.clone(),
            "Refresh Cloud Layers",
        ))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(duplicate_extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "view descriptor weather.cloud_layers already registered"
    );
    let operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    assert!(!operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array")
        .iter()
        .any(|operation| operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some(operation_path.as_str())));
}

#[test]
fn editor_runtime_rejects_duplicate_extension_menu_paths_without_registering_operations() {
    use crate::core::editor_event::{EditorEvent, MenuAction};
    use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationDescriptor, EditorOperationPath,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_duplicate_extension_menu");
    let first_operation = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let mut first_extension = EditorExtensionRegistry::default();
    first_extension
        .register_operation(
            EditorOperationDescriptor::new(first_operation.clone(), "Refresh Cloud Layers")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)),
        )
        .unwrap();
    first_extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            first_operation,
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(first_extension)
        .expect("register first extension menu");

    let second_operation = EditorOperationPath::parse("Weather.CloudLayer.Reset").unwrap();
    let mut duplicate_extension = EditorExtensionRegistry::default();
    duplicate_extension
        .register_operation(EditorOperationDescriptor::new(
            second_operation.clone(),
            "Reset Cloud Layers",
        ))
        .unwrap();
    duplicate_extension
        .register_menu_item(EditorMenuItemDescriptor::new(
            "Tools/Weather/Refresh Cloud Layers",
            second_operation.clone(),
        ))
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(duplicate_extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor menu item Tools/Weather/Refresh Cloud Layers already registered"
    );
    let operations = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    assert!(!operations
        .value
        .as_ref()
        .and_then(|value| value.get("operations"))
        .and_then(serde_json::Value::as_array)
        .expect("operations array")
        .iter()
        .any(|operation| operation
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            == Some(second_operation.as_str())));
}

#[test]
fn editor_runtime_rejects_component_drawer_bindings_to_missing_operations() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_component_drawer_missing_binding");
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                "weather.Component.CloudLayer",
                "asset://weather/editor/cloud_layer.inspector.ui.toml",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_binding("Weather.CloudLayer.Refresh"),
        )
        .unwrap();

    let error = runtime
        .runtime
        .register_editor_extension(extension)
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor operation Weather.CloudLayer.Refresh is not registered"
    );
}

#[test]
fn editor_extension_registry_rejects_invalid_component_drawer_operation_bindings() {
    use crate::core::editor_extension::{ComponentDrawerDescriptor, EditorExtensionRegistry};

    let mut extension = EditorExtensionRegistry::default();
    let error = extension
        .register_component_drawer(
            ComponentDrawerDescriptor::new(
                "weather.Component.CloudLayer",
                "asset://weather/editor/cloud_layer.inspector.ui.toml",
                "weather.editor.CloudLayerInspectorController",
            )
            .with_binding("Weather.Refresh"),
        )
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        "editor operation path `Weather.Refresh` is invalid"
    );
}

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
        .dispatch_envelope(slint_menu_action("CreateNode.Cube").unwrap())
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
        binding_record.operation_id.as_deref(),
        Some("Scene.Node.CreateCube")
    );
    assert_eq!(
        action.runtime.journal().records()[0].event,
        slint_record.event
    );
    assert_eq!(
        action.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("Scene.Node.CreateCube")
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
        .dispatch_envelope(slint_menu_action("CreateNode.Cube").unwrap())
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
fn slint_preset_menu_actions_normalize_to_layout_events_with_expected_names() {
    let save = slint_menu_action("SavePreset.rider").unwrap();
    let load = slint_menu_action("LoadPreset.").unwrap();

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
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: ui_asset_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open ui asset event");

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_path: ui_asset_path.to_string_lossy().into_owned(),
        })
    );
    assert!(record.effects.contains(&EditorEventEffect::LayoutChanged));
    assert!(runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .any(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset")));

    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn animation_binding_without_active_sequence_editor_reports_ignored_status_line() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_animation_binding");
    let binding = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "CreateTrackButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTrack {
            track_path: "Root/Hero:AnimationPlayer.weight".to_string(),
        }),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("animation binding should dispatch through runtime");

    assert_eq!(
        record.event,
        EditorEvent::Animation(EditorAnimationEvent::CreateTrack {
            track_path: AnimationTrackPath::parse("Root/Hero:AnimationPlayer.weight").unwrap(),
        })
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(record
        .effects
        .contains(&EditorEventEffect::ReflectionChanged));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Ignored animation command because active center tab is not an animation sequence editor"
    );
}

#[test]
fn animation_graph_and_state_machine_bindings_without_open_editor_report_ignored_status_line() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_animation_graph_binding");
    let binding = EditorUiBinding::new(
        "AnimationGraphEditorView",
        "CreateTransition",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTransition {
            state_machine_path: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 8,
        }),
    );

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .expect("graph/state-machine animation binding should dispatch through runtime");

    assert_eq!(
        record.event,
        EditorEvent::Animation(EditorAnimationEvent::CreateTransition {
            state_machine_path: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 8,
        })
    );
    assert!(record
        .effects
        .contains(&EditorEventEffect::PresentationChanged));
    assert!(record
        .effects
        .contains(&EditorEventEffect::ReflectionChanged));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        "Ignored animation command because active center tab is not an animation graph editor"
    );
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
    assert!(record.effects.contains(&EditorEventEffect::LayoutChanged));
    assert!(runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .any(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset")));

    let _ = fs::remove_file(ui_asset_path);
}

#[test]
fn asset_open_event_routes_animation_assets_to_animation_editor_views() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_animation_asset_open");
    let sequence_path =
        std::env::temp_dir().join("zircon_editor_event_animation_asset_open.sequence.zranim");
    let graph_path =
        std::env::temp_dir().join("zircon_editor_event_animation_asset_open.graph.zranim");
    let state_machine_path =
        std::env::temp_dir().join("zircon_editor_event_animation_asset_open.state_machine.zranim");
    fs::write(&sequence_path, b"").unwrap();
    fs::write(&graph_path, b"").unwrap();
    fs::write(&state_machine_path, b"").unwrap();

    let sequence_record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: sequence_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open animation sequence asset");
    assert!(sequence_record
        .effects
        .contains(&EditorEventEffect::LayoutChanged));

    let instances = runtime.runtime.current_view_instances();
    let sequence_view = instances
        .iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("animation sequence view should open");
    assert_eq!(
        sequence_view.serializable_payload["path"],
        json!(sequence_path.to_string_lossy().to_string())
    );

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: graph_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open animation graph asset");
    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: state_machine_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("open animation state machine asset");

    let graph_views = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .filter(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph")
        })
        .collect::<Vec<_>>();
    assert_eq!(graph_views.len(), 2);
    assert!(graph_views.iter().any(|instance| {
        instance.serializable_payload["path"] == json!(graph_path.to_string_lossy().to_string())
    }));
    assert!(graph_views.iter().any(|instance| {
        instance.serializable_payload["path"]
            == json!(state_machine_path.to_string_lossy().to_string())
    }));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        format!(
            "Opened animation graph editor for {}",
            state_machine_path.to_string_lossy()
        )
    );

    let _ = fs::remove_file(sequence_path);
    let _ = fs::remove_file(graph_path);
    let _ = fs::remove_file(state_machine_path);
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
            .contains(&EditorEventEffect::PresentationChanged));
    }
}
