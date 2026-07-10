use super::*;

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
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
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
                EditorOperationPath::parse("edit.history.undo").unwrap(),
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
        "scene.node.create_cube"
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("edit.history.redo").unwrap(),
            ),
        )
        .expect("redo operation");
    let stack_after_redo = runtime.runtime.operation_stack();
    assert_eq!(stack_after_redo.undo_stack().len(), 1);
    assert!(stack_after_redo.redo_stack().is_empty());
    assert_eq!(
        stack_after_redo.undo_stack()[0].operation_id.as_str(),
        "scene.node.create_cube"
    );

    let response = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::QueryOperationStack);
    let value = response.value.as_ref().expect("stack value");
    assert_eq!(
        value["undo_stack"][0]["operation_id"].as_str(),
        Some("scene.node.create_cube")
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
    let operation_path = EditorOperationPath::parse("scene.node.create_cube").unwrap();

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
        "scene.node.create_cube"
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
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
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
                EditorOperationPath::parse("edit.history.undo").unwrap(),
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
                EditorOperationPath::parse("edit.history.redo").unwrap(),
            ),
        )
        .expect("redo operation");
    assert_eq!(
        runtime.runtime.operation_stack().undo_stack()[0].source,
        EditorEventSource::Cli
    );
}

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
    assert!(runtime.runtime.operation_stack().undo_stack().is_empty());
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
    use zircon_runtime::plugin::{
        BridgeDiagnosticsMatrix, BridgeDiagnosticsSnapshot, BridgeInterfaceSnapshot,
        BridgeInterfaceStatus, BridgeTableDiagnosticsSummary, InterfaceSlot, PluginModuleId,
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
fn inspector_field_apply_batch_records_undoable_operation_stack_entry() {
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

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
        runtime.runtime.operation_stack().undo_stack()[0]
            .operation_id
            .as_str(),
        "inspector.field.apply_batch"
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Menu,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("edit.history.undo").unwrap(),
            ),
        )
        .expect("undo inspector apply batch");
    let stack_after_undo = runtime.runtime.operation_stack();
    assert!(stack_after_undo.undo_stack().is_empty());
    assert_eq!(
        stack_after_undo.redo_stack()[0].operation_id.as_str(),
        "inspector.field.apply_batch"
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
