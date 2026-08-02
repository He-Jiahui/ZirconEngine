use super::*;

#[test]
fn play_mode_menu_operations_use_plugin_activation_and_record_operation_identity() {
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use crate::core::play::{PluginBridgeActivation, PluginBridgeActivationReport};

    #[derive(Default)]
    struct RecordingActivation {
        calls: Mutex<Vec<String>>,
    }

    impl PluginBridgeActivation for RecordingActivation {
        fn activate(
            &self,
            project_root: Option<&Path>,
        ) -> Result<PluginBridgeActivationReport, String> {
            self.calls.lock().unwrap().push(format!(
                "enter:{}",
                project_root.is_some_and(|path| path.is_absolute())
            ));
            Ok(PluginBridgeActivationReport::default())
        }

        fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
            self.calls.lock().unwrap().push("exit".to_string());
            Ok(PluginBridgeActivationReport::default())
        }
    }

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_play_session_controller");
    let activation = Arc::new(RecordingActivation::default());
    runtime
        .runtime
        .set_plugin_bridge_activation(activation.clone());

    let enter_record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode),
        )
        .expect("enter play mode");
    assert_eq!(
        enter_record.operation_id.as_deref(),
        Some("runtime.play_mode.enter")
    );
    assert_eq!(
        runtime.runtime.editor_snapshot().session_mode,
        crate::ui::workbench::startup::EditorSessionMode::Playing
    );

    let exit_record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode),
        )
        .expect("exit play mode");
    assert_eq!(
        exit_record.operation_id.as_deref(),
        Some("runtime.play_mode.exit")
    );
    assert_eq!(
        runtime.runtime.editor_snapshot().session_mode,
        crate::ui::workbench::startup::EditorSessionMode::Project
    );
    assert_eq!(
        activation.calls.lock().unwrap().as_slice(),
        ["enter:true".to_string(), "exit".to_string()]
    );
}

#[test]
fn plugin_bridge_activation_matrix_projects_to_editor_snapshot() {
    use std::path::Path;
    use std::sync::Arc;

    use crate::core::play::{PluginBridgeActivation, PluginBridgeActivationReport};
    use zircon_runtime::core::framework::bridge::{
        BridgeDiagnosticsSnapshot, BridgeInterfaceStatus, InterfaceSlot,
    };
    use zircon_runtime::plugin::{
        BridgeDiagnosticsMatrix, BridgeInterfaceSnapshot, BridgeTableDiagnosticsSummary,
        PluginModuleId,
    };

    struct BridgeMatrixActivation;

    impl PluginBridgeActivation for BridgeMatrixActivation {
        fn activate(
            &self,
            _project_root: Option<&Path>,
        ) -> Result<PluginBridgeActivationReport, String> {
            Ok(PluginBridgeActivationReport {
                diagnostics: Vec::new(),
                bridge_diagnostics: Some(BridgeDiagnosticsMatrix {
                    summary: BridgeTableDiagnosticsSummary {
                        total_interfaces: 1,
                        enabled_interfaces: 1,
                        disabled_interfaces: 0,
                        installed_providers: 1,
                        missing_providers: 0,
                        enabled_calls: 3,
                        not_enabled_calls: 1,
                    },
                    rows: vec![BridgeInterfaceSnapshot {
                        slot: InterfaceSlot::from_raw(7),
                        interface_id: "physics.query.v1".to_string(),
                        owner: PluginModuleId::from_raw(2),
                        generation: 4,
                        provider_installed: true,
                        status: BridgeInterfaceStatus::Enabled,
                        diagnostics: BridgeDiagnosticsSnapshot {
                            enabled_calls: 3,
                            not_enabled_calls: 1,
                        },
                    }],
                }),
            })
        }

        fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
            Ok(PluginBridgeActivationReport::default())
        }
    }

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_bridge_matrix_backend");
    runtime
        .runtime
        .set_plugin_bridge_activation(Arc::new(BridgeMatrixActivation));

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::EnterPlayMode),
        )
        .expect("enter play mode");

    let bridge = runtime.runtime.editor_snapshot().bridge_diagnostics;
    assert_eq!(bridge.summary.total_interfaces, 1);
    assert_eq!(bridge.summary.enabled_calls, 3);
    assert_eq!(bridge.rows[0].interface_id, "physics.query.v1");
    assert_eq!(bridge.rows[0].owner_module_slot, 2);
    assert_eq!(bridge.rows[0].status, "Enabled");
    assert!(
        bridge
            .diagnostic_lines
            .iter()
            .any(|line| line.contains("bridge.interface"))
    );

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode),
        )
        .expect("exit play mode");
    assert!(
        runtime
            .runtime
            .editor_snapshot()
            .bridge_diagnostics
            .rows
            .is_empty()
    );
}

#[test]
fn inspector_field_apply_batch_records_operation_identity_without_synthetic_history() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_inspector_operation_stack");

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Inspector(EditorInspectorEvent {
                subject_path: "entity://selected".to_string(),
                changes: vec![InspectorFieldChange::new(
                    "name",
                    UiBindingValue::string("Operation Cube"),
                )],
            }),
        )
        .expect("inspector field commit");

    assert_eq!(
        record.operation_id.as_deref(),
        Some("inspector.field.apply_batch")
    );
    assert_eq!(
        record.operation_display_name.as_deref(),
        Some("Apply Inspector Changes")
    );
    assert_eq!(
        runtime.runtime.editor_snapshot().inspector.unwrap().name,
        "Operation Cube"
    );
    assert_eq!(
        runtime.runtime.journal().records()[0]
            .operation_id
            .as_deref(),
        Some("inspector.field.apply_batch")
    );
}

#[test]
fn inspector_binding_trace_records_path_and_transaction() {
    use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_inspector_binding_trace");
    let binding = EditorUiBinding::new(
        "Inspector",
        "TransformPositionXCommit",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            [InspectorFieldChange::new(
                "transform.translation.x",
                UiBindingValue::Float(42.0),
            )],
        ),
    );
    let binding_path = binding.path().native_prefix();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("inspector binding commit");

    assert_eq!(record.binding_path.as_deref(), Some(binding_path.as_str()));
    assert_eq!(
        record.operation_id.as_deref(),
        Some("inspector.field.apply_batch")
    );
    let transaction_id = record
        .transaction_id
        .expect("inspector must create a transaction");
    assert_eq!(record.save_generation, None);

    let serialized = serde_json::to_value(&record).expect("trace record should serialize");
    assert_eq!(serialized["binding_path"], serde_json::json!(binding_path));
    assert_eq!(
        serialized["transaction_id"],
        serde_json::json!(transaction_id)
    );
    assert!(serialized.get("save_generation").is_none());
}

#[test]
fn inspector_no_op_trace_does_not_reuse_a_prior_transaction_identity() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_inspector_no_op_trace");
    let event = EditorEvent::Inspector(EditorInspectorEvent {
        subject_path: "entity://selected".to_string(),
        changes: vec![InspectorFieldChange::new(
            "transform.translation.x",
            UiBindingValue::Float(42.0),
        )],
    });

    let committed = runtime
        .runtime
        .dispatch_event(EditorEventSource::RetainedHost, event.clone())
        .expect("initial inspector edit must commit a transaction");
    assert!(committed.transaction_id.is_some());

    let no_op = runtime
        .runtime
        .dispatch_event(EditorEventSource::RetainedHost, event)
        .expect("repeating the same inspector value must be accepted as a no-op");
    assert_eq!(no_op.transaction_id, None);
}

#[test]
fn operation_binding_trace_records_path() {
    use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_binding_trace");
    let binding = EditorUiBinding::new(
        "WindowMenu",
        "ResetLayout",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::editor_operation("window.layout.reset"),
    );
    let binding_path = binding.path().native_prefix();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("operation binding should dispatch");

    assert_eq!(record.binding_path.as_deref(), Some(binding_path.as_str()));
    assert_eq!(record.operation_id.as_deref(), Some("window.layout.reset"));
}

#[test]
fn operation_execution_trace_records_its_transaction_identity() {
    use crate::core::editor_event::EditorOperationEvent;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_operation_transaction_trace");

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Operation(EditorOperationEvent::CommandExecuted {
                operation_id: "scene.node.duplicate".to_string(),
                transaction_id: 17,
                group_open: false,
            }),
        )
        .expect("operation execution event should dispatch");

    assert_eq!(record.transaction_id, Some(17));
    assert_eq!(record.save_generation, None);
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
                EditorOperationPath::parse("window.layout.reset").unwrap(),
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
        Some("window.layout.reset")
    );
}
