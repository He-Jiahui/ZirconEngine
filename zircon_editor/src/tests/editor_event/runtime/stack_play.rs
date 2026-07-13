use super::*;

#[test]
fn play_mode_menu_operations_use_runtime_backend_and_record_operation_identity() {
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use crate::core::play::{EditorRuntimePlayModeBackend, EditorRuntimePlayModeBackendReport};

    #[derive(Default)]
    struct RecordingBackend {
        calls: Mutex<Vec<String>>,
    }

    impl EditorRuntimePlayModeBackend for RecordingBackend {
        fn enter_play_mode(
            &self,
            project_root: Option<&Path>,
        ) -> Result<EditorRuntimePlayModeBackendReport, String> {
            self.calls.lock().unwrap().push(format!(
                "enter:{}",
                project_root.is_some_and(|path| path.is_absolute())
            ));
            Ok(EditorRuntimePlayModeBackendReport::default())
        }

        fn exit_play_mode(&self) -> Result<EditorRuntimePlayModeBackendReport, String> {
            self.calls.lock().unwrap().push("exit".to_string());
            Ok(EditorRuntimePlayModeBackendReport::default())
        }
    }

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_play_mode_backend");
    let backend = Arc::new(RecordingBackend::default());
    runtime
        .runtime
        .set_runtime_play_mode_backend(backend.clone());

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
        backend.calls.lock().unwrap().as_slice(),
        ["enter:true".to_string(), "exit".to_string()]
    );
}

#[test]
fn play_mode_backend_bridge_matrix_projects_to_editor_snapshot() {
    use std::path::Path;
    use std::sync::Arc;

    use crate::core::play::{EditorRuntimePlayModeBackend, EditorRuntimePlayModeBackendReport};
    use zircon_runtime::core::framework::bridge::{
        BridgeDiagnosticsSnapshot, BridgeInterfaceStatus, InterfaceSlot,
    };
    use zircon_runtime::plugin::{
        BridgeDiagnosticsMatrix, BridgeInterfaceSnapshot, BridgeTableDiagnosticsSummary,
        PluginModuleId,
    };

    struct BridgeMatrixBackend;

    impl EditorRuntimePlayModeBackend for BridgeMatrixBackend {
        fn enter_play_mode(
            &self,
            _project_root: Option<&Path>,
        ) -> Result<EditorRuntimePlayModeBackendReport, String> {
            Ok(EditorRuntimePlayModeBackendReport {
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

        fn exit_play_mode(&self) -> Result<EditorRuntimePlayModeBackendReport, String> {
            Ok(EditorRuntimePlayModeBackendReport::default())
        }
    }

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_bridge_matrix_backend");
    runtime
        .runtime
        .set_runtime_play_mode_backend(Arc::new(BridgeMatrixBackend));

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
    assert!(bridge
        .diagnostic_lines
        .iter()
        .any(|line| line.contains("bridge.interface")));

    runtime
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode),
        )
        .expect("exit play mode");
    assert!(runtime
        .runtime
        .editor_snapshot()
        .bridge_diagnostics
        .rows
        .is_empty());
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
