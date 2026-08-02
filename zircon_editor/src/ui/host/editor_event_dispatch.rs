use crate::core::commands::{
    EditorCommandDescriptor, EditorCommandDispatchError, EditorCommandRegistry,
};
use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_event::{
    EditorEvent, EditorEventDispatcher, EditorEventEffect, EditorEventEnvelope,
    EditorEventListenerControlRequest, EditorEventListenerControlResponse, EditorEventRecord,
    EditorEventResult, EditorEventSource, EditorEventTransient, EditorOperationEvent, MenuAction,
};
use crate::core::editor_operation::{
    EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::binding_dispatch::editor_event_normalization::normalize_editor_event_binding;
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;
use crate::ui::workbench::snapshot::EditorConsoleMessageLevel;
use serde_json::{Number, Value};
use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventBinding};

use super::editor_event_execution::{event_result_value, execute_event, undo_policy_for_event};

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
    ) -> Result<EditorEventRecord, String> {
        self.dispatch_normalized_event_with_metadata(source, event, None, None)
    }

    pub(crate) fn dispatch_normalized_event_with_operation(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
        operation: Option<(EditorOperationPath, String, Value, Option<String>)>,
        binding_path: Option<String>,
    ) -> Result<EditorEventRecord, String> {
        self.dispatch_normalized_event_with_metadata(source, event, operation, binding_path)
    }

    fn dispatch_normalized_event_with_metadata(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
        operation: Option<(EditorOperationPath, String, Value, Option<String>)>,
        binding_path: Option<String>,
    ) -> Result<EditorEventRecord, String> {
        let stamp = self.context().events().begin_event();
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
                        .map(|descriptor| descriptor.display_name().to_string()),
                    None,
                    None,
                ),
            };

        let execution = match execute_event(self, &event) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.shell()
                    .lock()
                    .state
                    .set_status_line_with_level(error.clone(), EditorConsoleMessageLevel::Error);
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
                    result: EditorEventResult::failure(error.clone()),
                };
                self.refresh_workbench_for_effects(&effects);
                emit_mvp_authoring_product_trace(&record, "failed");
                self.context().events().record(record);
                return Err(error);
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
            result: EditorEventResult::success(event_result_value(
                stamp.after_revision,
                execution.changed(),
            )),
        };
        if execution.changed() {
            self.publish_scene_inspection_publication();
        }
        self.refresh_workbench_for_effects(execution.effects());
        emit_mvp_authoring_product_trace(&record, "completed");
        self.context().events().record(record.clone());
        Ok(record)
    }

    fn authoring_trace(&self, event: &EditorEvent, changed: bool) -> (Option<u64>, Option<u64>) {
        let transactions = self.context().transactions();
        match event {
            EditorEvent::Inspector(_) if changed => (
                transactions
                    .history_status(HistoryContextId::Global)
                    .ok()
                    .and_then(|history| history.top.map(|transaction| transaction.raw())),
                None,
            ),
            EditorEvent::Operation(EditorOperationEvent::CommandExecuted {
                transaction_id,
                ..
            }) => (Some(*transaction_id), None),
            EditorEvent::WorkbenchMenu(MenuAction::SaveProject) => (
                None,
                transactions
                    .history_generation_snapshot(HistoryContextId::Global)
                    .ok(),
            ),
            _ => (None, None),
        }
    }
}

fn emit_mvp_authoring_product_trace(record: &EditorEventRecord, result: &str) {
    let Some(event_kind) = mvp_authoring_trace_event_kind(&record.event) else {
        return;
    };
    write_log(
        "editor_authoring_trace",
        mvp_authoring_product_trace_diagnostic(
            result,
            event_kind,
            record.binding_path.as_deref(),
            record.operation_id.as_deref(),
            record.transaction_id,
            record.save_generation,
        ),
    );
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
    fn dispatch_envelope(
        &self,
        envelope: EditorEventEnvelope,
    ) -> Result<EditorEventRecord, String> {
        self.dispatch_normalized_event(envelope.source, envelope.event)
    }

    fn dispatch_binding(
        &self,
        binding: UiEventBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        let binding =
            EditorUiBinding::from_ui_binding(binding).map_err(|error| error.to_string())?;
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
        self.dispatch_normalized_event_with_metadata(
            source,
            event,
            None,
            Some(binding.path().native_prefix()),
        )
    }

    fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String> {
        self.dispatch_normalized_event(source, event)
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
    ) -> Result<Option<EditorEventRecord>, String> {
        match binding.payload() {
            EditorUiBindingPayload::EditorOperation {
                operation_id,
                arguments,
            } => {
                let invocation = operation_invocation(operation_id, arguments)?;
                self.invoke_operation_with_binding_path(
                    operation_source_for_event_source(source),
                    invocation,
                    Some(binding.path().native_prefix()),
                )
                .map(Some)
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
    ) -> Result<EditorEventRecord, String> {
        let command_id = {
            let commands = self.commands().lock();
            commands
                .command(command_id)
                .map(|command| command.id().clone())
        }
        .ok_or_else(|| {
            EditorCommandDispatchError::UnknownCommand(command_id.to_string()).to_string()
        })?;
        self.invoke_operation_with_binding_path(
            operation_source_for_event_source(source),
            EditorOperationInvocation::new(command_id),
            Some(binding_path),
        )
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
) -> Result<EditorOperationInvocation, String> {
    let operation_id =
        EditorOperationPath::parse(operation_id.to_string()).map_err(|error| error.to_string())?;
    Ok(EditorOperationInvocation::new(operation_id)
        .with_arguments(ui_binding_arguments_to_json(arguments)))
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
    match value {
        UiBindingValue::String(value) => Value::String(value.clone()),
        UiBindingValue::Unsigned(value) => Value::Number(Number::from(*value)),
        UiBindingValue::Signed(value) => Value::Number(Number::from(*value)),
        UiBindingValue::Float(value) => Number::from_f64(*value)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        UiBindingValue::Bool(value) => Value::Bool(*value),
        UiBindingValue::Null => Value::Null,
        UiBindingValue::Array(values) => {
            Value::Array(values.iter().map(ui_binding_value_to_json).collect())
        }
    }
}
