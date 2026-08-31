use crate::core::commands::{
    EditorCommandDescriptor, EditorCommandDispatchError, EditorCommandRegistry,
};
use crate::core::editor_event::{
    EditorAnimationEvent, EditorEvent, EditorEventDispatcher, EditorEventEffect,
    EditorEventEnvelope, EditorEventListenerControlRequest, EditorEventListenerControlResponse,
    EditorEventRecord, EditorEventResult, EditorEventSource, EditorEventTransient,
    EditorOperationEvent, MenuAction,
};
use crate::core::editor_operation::{
    EditorOperationInvocation, EditorOperationPath, EditorOperationPathError, EditorOperationSource,
};
use crate::core::logging::{EditorLogService, LogEntry, LogSeverity, LogSource};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingError, EditorUiBindingPayload};
use crate::ui::binding_dispatch::editor_event_normalization::{
    normalize_editor_event_binding, EditorEventNormalizationError,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::host::EditorOperationDispatchError;
use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;
use crate::ui::workbench::snapshot::EditorConsoleMessageLevel;
use serde_json::Value;
use thiserror::Error;
use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventBinding};

use super::editor_event_execution::{
    event_result_value, execute_event, undo_policy_for_event, EditorEventExecutionError,
};

#[derive(Debug, Error)]
pub enum EditorEventDispatchError {
    #[error(transparent)]
    Execution(#[from] EditorEventExecutionError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EventRecordPolicy {
    Durable,
    NativeCommandObservation,
}

impl EventRecordPolicy {
    fn advances_revision(self) -> bool {
        matches!(self, Self::Durable)
    }

    fn retain_result_in_journal(self) -> bool {
        matches!(self, Self::Durable)
    }

    fn retain_operation_arguments(self) -> bool {
        matches!(self, Self::Durable)
    }
}

#[derive(Debug, Error)]
pub enum EditorEventBindingDispatchError {
    #[error(transparent)]
    UiBinding(#[from] EditorUiBindingError),
    #[error(transparent)]
    OperationPath(#[from] EditorOperationPathError),
    #[error(transparent)]
    Command(#[from] EditorCommandDispatchError),
    #[error(transparent)]
    Normalization(#[from] EditorEventNormalizationError),
    #[error(transparent)]
    Operation(#[from] EditorOperationDispatchError),
    #[error(transparent)]
    EventDispatch(#[from] EditorEventDispatchError),
}

#[derive(Debug, Error)]
pub enum EditorEventDispatcherError {
    #[error(transparent)]
    Binding(#[from] EditorEventBindingDispatchError),
    #[error(transparent)]
    Event(#[from] EditorEventDispatchError),
}

impl EditorHostEventController {
    pub fn handle_event_listener_control_request(
        &self,
        request: EditorEventListenerControlRequest,
    ) -> EditorEventListenerControlResponse {
        self.context()
            .events()
            .handle_listener_control_request(request)
    }

    fn dispatch_normalized_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, EditorEventDispatchError> {
        self.dispatch_normalized_event_with_metadata(
            source,
            event,
            None,
            None,
            None,
            EventRecordPolicy::Durable,
        )
    }

    pub(crate) fn dispatch_normalized_event_with_operation(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
        operation: Option<(EditorOperationPath, String, Value, Option<String>)>,
        binding_path: Option<String>,
    ) -> Result<EditorEventRecord, EditorEventDispatchError> {
        self.dispatch_normalized_event_with_metadata(
            source,
            event,
            operation,
            binding_path,
            None,
            EventRecordPolicy::Durable,
        )
    }

    pub(crate) fn dispatch_normalized_native_result(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
        operation: Option<(EditorOperationPath, String, Value, Option<String>)>,
        binding_path: Option<String>,
        result: EditorEventResult,
    ) -> Result<EditorEventRecord, EditorEventDispatchError> {
        self.dispatch_normalized_event_with_metadata(
            source,
            event,
            operation,
            binding_path,
            Some(result),
            EventRecordPolicy::NativeCommandObservation,
        )
    }

    fn dispatch_normalized_event_with_metadata(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
        operation: Option<(EditorOperationPath, String, Value, Option<String>)>,
        binding_path: Option<String>,
        result_override: Option<EditorEventResult>,
        record_policy: EventRecordPolicy,
    ) -> Result<EditorEventRecord, EditorEventDispatchError> {
        let stamp = if record_policy.advances_revision() {
            self.context().events().begin_event()
        } else {
            self.context().events().begin_observation()
        };
        let undo_policy = undo_policy_for_event(&event);
        let registry_operation = if operation.is_none() {
            let operations = self.commands().lock();
            operations
                .descriptor_for_event(&event)
                .cloned()
                .or_else(|| dynamic_operation_for_event(&operations, &event))
        } else {
            None
        };
        let i18n = self.context().i18n();
        let locale = i18n.active_locale();
        let (operation_id, operation_display_name, operation_arguments, operation_group) =
            match operation {
                Some((operation_id, operation_display_name, arguments, group)) => (
                    Some(operation_id.to_string()),
                    Some(operation_display_name),
                    operation_arguments_for_record(arguments),
                    group,
                ),
                None => (
                    registry_operation
                        .as_ref()
                        .map(|descriptor| descriptor.id().to_string()),
                    registry_operation
                        .as_ref()
                        .map(|descriptor| descriptor.localized_label(i18n, &locale).to_string()),
                    None,
                    None,
                ),
            };

        let execution = match execute_event(self, &event) {
            Ok(outcome) => outcome,
            Err(error) => {
                let error_message = error.to_string();
                self.shell().lock().state.set_status_line_with_level(
                    error_message.clone(),
                    EditorConsoleMessageLevel::Error,
                );
                let effects = failure_effects_for_event(&event);
                let record = EditorEventRecord {
                    event_id: stamp.event_id,
                    sequence: stamp.sequence,
                    source,
                    event,
                    binding_path: binding_path.clone(),
                    operation_id: operation_id.clone(),
                    operation_display_name: operation_display_name.clone(),
                    operation_arguments: operation_arguments.clone(),
                    operation_group: operation_group.clone(),
                    transaction_id: None,
                    save_generation: None,
                    effects: effects.clone(),
                    undo_policy,
                    before_revision: stamp.before_revision,
                    after_revision: stamp.after_revision,
                    result: EditorEventResult::failure(error_message),
                };
                self.refresh_workbench_for_event_record(&record);
                emit_mvp_authoring_product_trace(self.context().logs(), &record, "failed");
                emit_failed_event_log(self.context().logs(), &record);
                self.context().events().record(record);
                return Err(error.into());
            }
        };

        let (transaction_id, save_generation) = self.authoring_trace(&event, execution.changed());
        let record = EditorEventRecord {
            event_id: stamp.event_id,
            sequence: stamp.sequence,
            source,
            event,
            binding_path,
            operation_id,
            operation_display_name,
            operation_arguments,
            operation_group,
            transaction_id,
            save_generation,
            effects: execution.effects().to_vec(),
            undo_policy,
            before_revision: stamp.before_revision,
            after_revision: stamp.after_revision,
            result: result_override.unwrap_or_else(|| {
                EditorEventResult::success(event_result_value(
                    stamp.after_revision,
                    execution.changed(),
                ))
            }),
        };
        if execution.changed() {
            self.publish_scene_inspection_publication();
        }
        self.refresh_workbench_for_event_record(&record);
        emit_mvp_authoring_product_trace(self.context().logs(), &record, "completed");
        let journal_record = if record_policy.retain_result_in_journal() {
            record.clone()
        } else {
            let mut journal_record = record.clone();
            journal_record.result = EditorEventResult::default();
            if !record_policy.retain_operation_arguments() {
                journal_record.operation_arguments = None;
            }
            journal_record
        };
        self.context().events().record(journal_record);
        Ok(record)
    }

    fn authoring_trace(&self, event: &EditorEvent, changed: bool) -> (Option<u64>, Option<u64>) {
        let transactions = self.context().transactions();
        let scene_history_context = self.shell().lock().state.active_scene_history_context();
        match event {
            EditorEvent::Inspector(_) if changed => (
                scene_history_context
                    .and_then(|history| transactions.history_status(history).ok())
                    .and_then(|history| history.top.map(|transaction| transaction.raw())),
                None,
            ),
            EditorEvent::Animation(event)
                if changed && animation_event_commits_document_transaction(event) =>
            {
                (
                    self.shell()
                        .lock()
                        .manager
                        .focused_animation_history_status()
                        .and_then(|history| history.top.map(|transaction| transaction.raw())),
                    None,
                )
            }
            EditorEvent::Operation(EditorOperationEvent::CommandExecuted {
                transaction_id,
                ..
            }) => (Some(*transaction_id), None),
            EditorEvent::Operation(EditorOperationEvent::NativeCommandExecuted { .. }) => {
                (None, None)
            }
            EditorEvent::WorkbenchMenu(MenuAction::SaveProject) => (
                None,
                scene_history_context
                    .and_then(|history| transactions.history_generation_snapshot(history).ok()),
            ),
            _ => (None, None),
        }
    }
}

fn animation_event_commits_document_transaction(event: &EditorAnimationEvent) -> bool {
    !matches!(
        event,
        EditorAnimationEvent::ScrubTimeline { .. }
            | EditorAnimationEvent::SetTimelineRange { .. }
            | EditorAnimationEvent::SelectTimelineSpan { .. }
            | EditorAnimationEvent::SetPlayback { .. }
    )
}

// Editor-event dispatch is not tied to a retained-host render frame. The log record sequence
// remains its ordering source, so use zero instead of misrepresenting an event sequence as a frame.
const UNKNOWN_EDITOR_EVENT_LOG_FRAME: u64 = 0;

fn emit_failed_event_log(logs: &EditorLogService, record: &EditorEventRecord) {
    let Some(error) = record.result.error.as_deref().map(str::trim) else {
        return;
    };
    let subject = record
        .operation_id
        .as_deref()
        .or(record.binding_path.as_deref())
        .unwrap_or("unattributed");
    let entry = LogEntry::new(
        LogSource::editor(),
        LogSeverity::Error,
        format!("Editor event `{subject}` failed: {error}"),
        UNKNOWN_EDITOR_EVENT_LOG_FRAME,
        None,
    )
    .or_else(|_| {
        LogEntry::new(
            LogSource::editor(),
            LogSeverity::Error,
            format!(
                "Editor event {} failed; diagnostic exceeds the log-entry limit.",
                record.sequence.0
            ),
            UNKNOWN_EDITOR_EVENT_LOG_FRAME,
            None,
        )
    });
    if let Ok(entry) = entry {
        let _ = logs.emit(entry);
    }
}

fn emit_mvp_authoring_product_trace(
    logs: &EditorLogService,
    record: &EditorEventRecord,
    result: &str,
) {
    let Some(event_kind) = mvp_authoring_trace_event_kind(&record.event) else {
        return;
    };
    let entry = LogEntry::new(
        LogSource::editor(),
        LogSeverity::Info,
        mvp_authoring_product_trace_diagnostic(
            result,
            event_kind,
            record.binding_path.as_deref(),
            record.operation_id.as_deref(),
            record.transaction_id,
            record.save_generation,
        ),
        UNKNOWN_EDITOR_EVENT_LOG_FRAME,
        None,
    )
    .or_else(|_| {
        LogEntry::new(
            LogSource::editor(),
            LogSeverity::Info,
            format!(
                "editor_authoring_trace result={result} event={event_kind} sequence={} diagnostic exceeds the log-entry limit.",
                record.sequence.0
            ),
            UNKNOWN_EDITOR_EVENT_LOG_FRAME,
            None,
        )
    });
    if let Ok(entry) = entry {
        let _ = logs.emit(entry);
    }
}

fn mvp_authoring_trace_event_kind(event: &EditorEvent) -> Option<&'static str> {
    match event {
        EditorEvent::Selection(_) => Some("selection"),
        EditorEvent::Inspector(_) => Some("inspector"),
        EditorEvent::WorkbenchMenu(MenuAction::SaveProject) => Some("save_project"),
        _ => None,
    }
}

fn mvp_authoring_product_trace_diagnostic(
    result: &str,
    event_kind: &str,
    binding_path: Option<&str>,
    operation_id: Option<&str>,
    transaction_id: Option<u64>,
    save_generation: Option<u64>,
) -> String {
    let transaction_id = transaction_id
        .map(|transaction_id| transaction_id.to_string())
        .unwrap_or_else(|| "none".to_string());
    let save_generation = save_generation
        .map(|generation| generation.to_string())
        .unwrap_or_else(|| "none".to_string());
    format!(
        "editor_authoring_trace result={result} event={event_kind} binding={} operation={} transaction_id={transaction_id} save_generation={save_generation}",
        binding_path.unwrap_or("unbound"),
        operation_id.unwrap_or("unresolved"),
    )
}

fn failure_effects_for_event(event: &EditorEvent) -> Vec<EditorEventEffect> {
    let mut effects = vec![
        EditorEventEffect::PresentationChanged,
        EditorEventEffect::ReflectionChanged,
    ];
    if matches!(event, EditorEvent::Viewport(_)) {
        effects.push(EditorEventEffect::RenderChanged);
    }
    effects
}

#[cfg(test)]
mod failure_effect_tests {
    use super::*;

    #[test]
    fn viewport_failure_invalidates_render_and_presentation() {
        let effects = failure_effects_for_event(&EditorEvent::Viewport(
            crate::core::editor_event::EditorViewportEvent::LeftReleased,
        ));

        assert!(effects.contains(&EditorEventEffect::RenderChanged));
        assert!(effects.contains(&EditorEventEffect::PresentationChanged));
        assert!(effects.contains(&EditorEventEffect::ReflectionChanged));
    }

    #[test]
    fn mvp_authoring_trace_keeps_binding_operation_and_generation_correlation() {
        let trace = mvp_authoring_product_trace_diagnostic(
            "completed",
            "inspector",
            Some("Inspector/TransformPositionXCommit"),
            Some("inspector.transform.position.x.commit"),
            Some(42),
            Some(8),
        );

        assert!(trace.contains("result=completed"));
        assert!(trace.contains("event=inspector"));
        assert!(trace.contains("binding=Inspector/TransformPositionXCommit"));
        assert!(trace.contains("operation=inspector.transform.position.x.commit"));
        assert!(trace.contains("transaction_id=42"));
        assert!(trace.contains("save_generation=8"));
        assert_eq!(
            mvp_authoring_trace_event_kind(&EditorEvent::WorkbenchMenu(MenuAction::SaveProject)),
            Some("save_project")
        );
    }
}

#[cfg(test)]
mod failure_log_tests {
    use super::emit_failed_event_log;
    use crate::core::editor_event::{
        EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord, EditorEventResult,
        EditorEventSequence, EditorEventSource, EditorEventUndoPolicy, MenuAction,
    };
    use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

    fn failed_record(error: impl Into<String>) -> EditorEventRecord {
        EditorEventRecord {
            event_id: EditorEventId::new(7),
            sequence: EditorEventSequence::new(11),
            source: EditorEventSource::RetainedHost,
            event: EditorEvent::WorkbenchMenu(MenuAction::SaveProject),
            binding_path: Some("WorkbenchMenu/SaveProject".to_string()),
            operation_id: Some("project.save".to_string()),
            operation_display_name: Some("Save Project".to_string()),
            operation_arguments: None,
            operation_group: None,
            transaction_id: None,
            save_generation: None,
            effects: vec![EditorEventEffect::PresentationChanged],
            undo_policy: EditorEventUndoPolicy::NonUndoable,
            before_revision: 4,
            after_revision: 4,
            result: EditorEventResult::failure(error),
        }
    }

    #[test]
    fn failed_editor_event_emits_a_structured_error_log() {
        let logs = EditorLogService::default();

        emit_failed_event_log(&logs, &failed_record("disk is read-only"));

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 1);
        let entry = records[0].entry();
        assert_eq!(entry.source(), &LogSource::editor());
        assert_eq!(entry.severity(), LogSeverity::Error);
        assert_eq!(entry.timestamp_frame(), 0);
        assert_eq!(
            entry.message(),
            "Editor event `project.save` failed: disk is read-only"
        );
    }

    #[test]
    fn oversized_editor_event_diagnostic_uses_a_bounded_fallback_log() {
        let logs = EditorLogService::default();
        let oversized_error = "x".repeat(9 * 1024);

        emit_failed_event_log(&logs, &failed_record(oversized_error));

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].entry().message(),
            "Editor event 11 failed; diagnostic exceeds the log-entry limit."
        );
    }

    #[test]
    fn authoring_trace_uses_the_editor_log_service() {
        let logs = EditorLogService::default();
        let record = failed_record("disk is read-only");

        super::emit_mvp_authoring_product_trace(&logs, &record, "failed");

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 1);
        let entry = records[0].entry();
        assert_eq!(entry.source(), &LogSource::editor());
        assert_eq!(entry.severity(), LogSeverity::Info);
        assert_eq!(entry.timestamp_frame(), 0);
        assert_eq!(
            entry.message(),
            "editor_authoring_trace result=failed event=save_project binding=WorkbenchMenu/SaveProject operation=project.save transaction_id=none save_generation=none"
        );
    }

    #[test]
    fn oversized_authoring_trace_uses_a_bounded_fallback_log() {
        let logs = EditorLogService::default();
        let mut record = failed_record("disk is read-only");
        record.binding_path = Some("x".repeat(9 * 1024));

        super::emit_mvp_authoring_product_trace(&logs, &record, "failed");

        let records = logs.snapshot(&LogFilter::default());
        assert_eq!(records.len(), 1);
        let entry = records[0].entry();
        assert_eq!(entry.source(), &LogSource::editor());
        assert_eq!(entry.severity(), LogSeverity::Info);
        assert_eq!(
            entry.message(),
            "editor_authoring_trace result=failed event=save_project sequence=11 diagnostic exceeds the log-entry limit."
        );
    }
}

fn dynamic_operation_for_event(
    registry: &EditorCommandRegistry,
    event: &EditorEvent,
) -> Option<EditorCommandDescriptor> {
    let path = match event {
        EditorEvent::Inspector(_) => "inspector.field.apply_batch",
        _ => return None,
    };
    let path = EditorOperationPath::parse(path).ok()?;
    registry.command(path.as_str()).cloned()
}

fn operation_arguments_for_record(arguments: Value) -> Option<Value> {
    match arguments {
        Value::Null => None,
        Value::Array(values) if values.is_empty() => None,
        other => Some(other),
    }
}

impl EditorEventDispatcher for EditorHostEventController {
    type Error = EditorEventDispatcherError;

    fn dispatch_envelope(
        &self,
        envelope: EditorEventEnvelope,
    ) -> Result<EditorEventRecord, Self::Error> {
        self.dispatch_normalized_event(envelope.source, envelope.event)
            .map_err(EditorEventDispatcherError::from)
    }

    fn dispatch_binding(
        &self,
        binding: UiEventBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, Self::Error> {
        self.dispatch_binding_typed(binding, source)
            .map_err(EditorEventDispatcherError::from)
    }

    fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, Self::Error> {
        self.dispatch_normalized_event(source, event)
            .map_err(EditorEventDispatcherError::from)
    }
}

impl EditorHostEventController {
    pub(crate) fn dispatch_binding_typed(
        &self,
        binding: UiEventBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, EditorEventBindingDispatchError> {
        let binding = EditorUiBinding::from_ui_binding(binding)?;
        if is_material_component_lab_binding(&binding) {
            return Ok(self.record_material_component_lab_feedback(source, &binding));
        }
        if let Some(action_id) = component_lab_preview_action_id(&binding) {
            return Ok(self.record_component_lab_preview_action(source, &binding, action_id));
        }
        if let Some(record) = self.dispatch_operation_binding(&binding, source.clone())? {
            return Ok(record);
        }
        let context = self.context().command_eval().snapshot();
        let event = {
            let commands = self.commands().lock();
            normalize_editor_event_binding(&binding, &commands, &context)?
        };
        Ok(self.dispatch_normalized_event_with_metadata(
            source,
            event,
            None,
            Some(binding.path().native_prefix()),
            None,
            EventRecordPolicy::Durable,
        )?)
    }
}

fn is_material_component_lab_binding(binding: &EditorUiBinding) -> bool {
    binding
        .as_ui_binding()
        .action
        .as_ref()
        .is_some_and(|call| call.symbol == "MaterialComponentLab")
}

fn component_lab_preview_action_id(binding: &EditorUiBinding) -> Option<&str> {
    match binding.payload() {
        crate::ui::binding::EditorUiBindingPayload::MenuAction { action_id }
            if is_workbench_preview_action(action_id) =>
        {
            Some(action_id.as_str())
        }
        _ => None,
    }
}

impl EditorHostEventController {
    fn dispatch_operation_binding(
        &self,
        binding: &EditorUiBinding,
        source: EditorEventSource,
    ) -> Result<Option<EditorEventRecord>, EditorEventBindingDispatchError> {
        match binding.payload() {
            EditorUiBindingPayload::EditorOperation {
                operation_id,
                arguments,
            } => {
                let invocation = operation_invocation(operation_id, arguments)?;
                Ok(Some(self.invoke_operation_with_binding_path(
                    operation_source_for_event_source(source),
                    invocation,
                    Some(binding.path().native_prefix()),
                )?))
            }
            EditorUiBindingPayload::EditorCommand { command_id } => self
                .dispatch_editor_command_binding(command_id, source, binding.path().native_prefix())
                .map(Some),
            _ => Ok(None),
        }
    }

    fn dispatch_editor_command_binding(
        &self,
        command_id: &str,
        source: EditorEventSource,
        binding_path: String,
    ) -> Result<EditorEventRecord, EditorEventBindingDispatchError> {
        let command_id = {
            let commands = self.commands().lock();
            registered_command_path(&commands, command_id)?
        };
        Ok(self.invoke_operation_with_binding_path(
            operation_source_for_event_source(source),
            EditorOperationInvocation::new(command_id),
            Some(binding_path),
        )?)
    }

    fn record_material_component_lab_feedback(
        &self,
        source: EditorEventSource,
        binding: &EditorUiBinding,
    ) -> EditorEventRecord {
        let stamp = self.context().events().begin_observation();
        let node_path = binding_node_path(binding);
        let event = EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path,
            pressed: false,
        });
        let undo_policy = undo_policy_for_event(&event);
        let record = EditorEventRecord {
            event_id: stamp.event_id,
            sequence: stamp.sequence,
            source,
            event,
            binding_path: Some(binding.path().native_prefix()),
            operation_id: None,
            operation_display_name: Some("Material Component Lab Feedback".to_string()),
            operation_arguments: None,
            operation_group: Some("MaterialComponentLab".to_string()),
            transaction_id: None,
            save_generation: None,
            effects: Vec::new(),
            undo_policy,
            before_revision: stamp.before_revision,
            after_revision: stamp.after_revision,
            result: EditorEventResult::success(event_result_value(stamp.after_revision, false)),
        };
        self.context().events().record(record.clone());
        record
    }

    fn record_component_lab_preview_action(
        &self,
        source: EditorEventSource,
        binding: &EditorUiBinding,
        action_id: &str,
    ) -> EditorEventRecord {
        let stamp = self.context().events().begin_observation();
        let node_path = component_lab_preview_node_path(binding, action_id);
        let event = EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path: node_path.clone(),
            pressed: false,
        });
        let undo_policy = undo_policy_for_event(&event);
        let record = EditorEventRecord {
            event_id: stamp.event_id,
            sequence: stamp.sequence,
            source,
            event,
            binding_path: Some(binding.path().native_prefix()),
            operation_id: None,
            operation_display_name: Some("Component Lab Preview Action".to_string()),
            operation_arguments: Some(serde_json::json!({
                "control_id": binding.path().control_id.clone(),
                "node_path": node_path,
                "action_id": action_id,
            })),
            operation_group: Some("ComponentLabPreview".to_string()),
            transaction_id: None,
            save_generation: None,
            effects: Vec::new(),
            undo_policy,
            before_revision: stamp.before_revision,
            after_revision: stamp.after_revision,
            result: EditorEventResult::success(event_result_value(stamp.after_revision, false)),
        };
        self.context().events().record(record.clone());
        record
    }
}

fn binding_node_path(binding: &EditorUiBinding) -> String {
    format!("{}/{}", binding.path().view_id, binding.path().control_id)
}

fn component_lab_preview_node_path(binding: &EditorUiBinding, action_id: &str) -> String {
    match action_id {
        "component_lab.input_dropdown.select" | "component_lab.button_dropdown.select" => {
            action_id.to_string()
        }
        _ => binding_node_path(binding),
    }
}

fn operation_invocation(
    operation_id: &str,
    arguments: &[UiBindingValue],
) -> Result<EditorOperationInvocation, EditorOperationPathError> {
    let operation_id = EditorOperationPath::parse(operation_id.to_string())?;
    Ok(EditorOperationInvocation::new(operation_id)
        .with_arguments(ui_binding_arguments_to_json(arguments)))
}

fn registered_command_path(
    commands: &EditorCommandRegistry,
    command_id: &str,
) -> Result<EditorOperationPath, EditorCommandDispatchError> {
    commands
        .command(command_id)
        .map(|command| command.id().clone())
        .ok_or_else(|| EditorCommandDispatchError::UnknownCommand(command_id.to_string()))
}

fn operation_source_for_event_source(source: EditorEventSource) -> EditorOperationSource {
    match source {
        EditorEventSource::Cli => EditorOperationSource::Cli,
        EditorEventSource::Headless | EditorEventSource::Mcp => EditorOperationSource::Remote,
        EditorEventSource::RetainedHost | EditorEventSource::Replay => {
            EditorOperationSource::UiBinding
        }
    }
}

fn ui_binding_arguments_to_json(arguments: &[UiBindingValue]) -> Value {
    if arguments.is_empty() {
        return Value::Null;
    }
    Value::Array(arguments.iter().map(ui_binding_value_to_json).collect())
}

fn ui_binding_value_to_json(value: &UiBindingValue) -> Value {
    value.to_json_value()
}

#[cfg(test)]
mod binding_dispatch_error_tests {
    use super::{operation_invocation, registered_command_path};
    use crate::core::commands::{EditorCommandDispatchError, EditorCommandRegistry};
    use crate::core::editor_operation::EditorOperationPathError;

    #[test]
    fn operation_binding_preserves_invalid_operation_path() {
        let error = operation_invocation("not a valid operation", &[])
            .expect_err("operation id with whitespace must be rejected");

        assert_eq!(
            error,
            EditorOperationPathError::InvalidOperationPath("not a valid operation".to_string())
        );
    }

    #[test]
    fn editor_command_binding_preserves_unknown_command() {
        let error =
            registered_command_path(&EditorCommandRegistry::default(), "scene.node.missing")
                .expect_err("unregistered editor command must be rejected");

        assert_eq!(
            error,
            EditorCommandDispatchError::UnknownCommand("scene.node.missing".to_string())
        );
    }
}
