use crate::core::commands::{
    EditorCommandDescriptor, EditorCommandDispatchError, EditorCommandRegistry,
};
use crate::core::editor_event::{
    DocumentCloseRevision, EditorAnimationEvent, EditorEvent, EditorEventDispatcher,
    EditorEventEffect, EditorEventEnvelope, EditorEventListenerControlRequest,
    EditorEventListenerControlResponse, EditorEventRecord, EditorEventResult, EditorEventSource,
    EditorEventTransient, EditorOperationEvent, EditorViewportEvent, MenuAction,
};
use crate::core::editor_message::DocumentId;
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
use crate::ui::workbench::event::core_layout_command_from_ui;
use crate::ui::workbench::layout::{LayoutCommand as UiLayoutCommand, MainPageId};
use crate::ui::workbench::snapshot::EditorConsoleMessageLevel;
use crate::ui::workbench::view::ViewInstanceId;
use serde_json::Value;
use thiserror::Error;
use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventBinding};

use super::editor_event_execution::{
    event_result_value, execute_event, undo_policy_for_event, EditorEventExecutionError,
    ExecutionOutcome,
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
    #[error("viewport binding target `{view_id:?}` is stale or retired")]
    StaleViewportView {
        view_id: crate::core::editor_event::ViewInstanceId,
    },
}

#[derive(Debug, Error)]
pub enum EditorEventDispatcherError {
    #[error(transparent)]
    Binding(#[from] EditorEventBindingDispatchError),
    #[error(transparent)]
    Event(#[from] EditorEventDispatchError),
}

impl EditorHostEventController {
    pub(crate) fn dispatch_authorized_close_views(
        &self,
        window_id: MainPageId,
        instance_ids: Vec<ViewInstanceId>,
        discard: Vec<(ViewInstanceId, DocumentId, DocumentCloseRevision)>,
    ) -> Result<EditorEventRecord, EditorEventDispatchError> {
        let event = EditorEvent::Layout(core_layout_command_from_ui(UiLayoutCommand::CloseViews {
            window_id: window_id.clone(),
            instance_ids: instance_ids.clone(),
        }));
        self.dispatch_normalized_event_with_metadata_using(
            EditorEventSource::RetainedHost,
            event,
            None,
            None,
            None,
            EventRecordPolicy::Durable,
            move |controller, _| {
                let changed = controller
                    .shell()
                    .lock()
                    .manager
                    .close_views_with_discard(&window_id, &instance_ids, &discard)
                    .map_err(|source| EditorEventExecutionError::Layout { source })?;
                Ok(ExecutionOutcome {
                    changed,
                    effects: vec![
                        EditorEventEffect::LayoutChanged,
                        EditorEventEffect::PresentationChanged,
                        EditorEventEffect::ReflectionChanged,
                    ],
                })
            },
        )
    }

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
        self.dispatch_normalized_event_with_metadata_using(
            source,
            event,
            operation,
            binding_path,
            result_override,
            record_policy,
            execute_event,
        )
    }

    fn dispatch_normalized_event_with_metadata_using(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
        operation: Option<(EditorOperationPath, String, Value, Option<String>)>,
        binding_path: Option<String>,
        result_override: Option<EditorEventResult>,
        record_policy: EventRecordPolicy,
        execute: impl FnOnce(
            &EditorHostEventController,
            &EditorEvent,
        ) -> Result<ExecutionOutcome, EditorEventExecutionError>,
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

        let execution = match execute(self, &event) {
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
#[path = "tests/editor_event_dispatch_failure_effect_tests.rs"]
mod failure_effect_tests;

#[cfg(test)]
#[path = "tests/editor_event_dispatch_failure_log_tests.rs"]
mod failure_log_tests;

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

    /// Dispatch a retained viewport binding for the committed Scene leaf that
    /// produced the surface event.  This deliberately reuses normalization,
    /// journal recording, and the ordinary executor instead of mutating a
    /// retained callback/session directly.
    pub(crate) fn dispatch_binding_typed_for_view(
        &self,
        binding: UiEventBinding,
        source: EditorEventSource,
        view_id: crate::core::editor_event::ViewInstanceId,
    ) -> Result<EditorEventRecord, EditorEventBindingDispatchError> {
        let live = {
            let shell = self.shell().lock();
            let manager_view_id =
                crate::ui::workbench::view::ViewInstanceId::new(view_id.0.clone());
            let manager_live = shell
                .manager
                .view_instance_ids_for_descriptor_key("editor.scene")
                .iter()
                .any(|candidate| candidate == &manager_view_id);
            manager_live
                && shell
                    .state
                    .viewport_controller
                    .session_if_live(&view_id)
                    .is_some()
        };
        if !live {
            return Err(EditorEventBindingDispatchError::StaleViewportView { view_id });
        }
        let binding = EditorUiBinding::from_ui_binding(binding)?;
        let context = self.context().command_eval().snapshot();
        let event = {
            let commands = self.commands().lock();
            normalize_editor_event_binding(&binding, &commands, &context)?
        };
        let EditorEvent::Viewport(event) = event else {
            return Err(EditorEventBindingDispatchError::Normalization(
                EditorEventNormalizationError::UnsupportedBinding {
                    native_binding: binding.native_binding(),
                },
            ));
        };
        Ok(self.dispatch_normalized_event_with_metadata(
            source,
            EditorEvent::Viewport(EditorViewportEvent::ForView {
                view_id,
                event: Box::new(event),
            }),
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
#[path = "tests/editor_event_dispatch_binding_dispatch_error_tests.rs"]
mod binding_dispatch_error_tests;
