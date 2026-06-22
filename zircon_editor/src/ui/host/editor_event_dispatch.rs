use crate::core::editor_event::{
    EditorEvent, EditorEventDispatcher, EditorEventEffect, EditorEventEnvelope, EditorEventId,
    EditorEventRecord, EditorEventResult, EditorEventRuntime, EditorEventSequence,
    EditorEventSource, EditorEventTransient, MenuAction,
};
use crate::core::editor_operation::{
    EditorOperationDescriptor, EditorOperationInvocation, EditorOperationPath,
    EditorOperationSource, EditorOperationStackEntry,
};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use crate::ui::binding_dispatch::editor_event_normalization::normalize_editor_event_binding;
use crate::ui::host::{EditorCommandAction, EditorCommandDispatchError, EditorCommandRegistry};
use crate::ui::retained_host::workbench_preview_actions::is_workbench_preview_action;
use crate::ui::workbench::model::operation_path_for_menu_action;
use serde_json::{Number, Value};
use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventBinding};

use super::editor_event_execution::{event_result_value, execute_event, undo_policy_for_event};

impl EditorEventRuntime {
    fn dispatch_normalized_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String> {
        self.dispatch_normalized_event_with_operation(source, event, None)
    }

    pub(crate) fn dispatch_normalized_event_with_operation(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
        operation: Option<(EditorOperationPath, String, bool, Value, Option<String>)>,
    ) -> Result<EditorEventRecord, String> {
        let mut inner = self.lock_inner();
        inner.next_event_id += 1;
        inner.next_sequence += 1;

        let before_revision = inner.revision;
        let after_revision = before_revision + 1;
        inner.revision = after_revision;

        let event_id = EditorEventId::new(inner.next_event_id);
        let sequence = EditorEventSequence::new(inner.next_sequence);
        let undo_policy = undo_policy_for_event(&event);
        let registry_operation = if operation.is_none() {
            inner
                .operation_registry
                .descriptor_for_event(&event)
                .cloned()
                .or_else(|| dynamic_operation_for_event(&inner, &event))
        } else {
            None
        };
        let (
            operation_id,
            operation_display_name,
            operation_arguments,
            operation_group,
            explicit_stack_entry,
        ) = match operation {
            Some((operation_id, operation_display_name, undoable, arguments, group)) => {
                let stack_entry = undoable.then(|| {
                    (
                        operation_id.clone(),
                        operation_display_name.clone(),
                        group.clone(),
                    )
                });
                (
                    Some(operation_id.to_string()),
                    Some(operation_display_name),
                    operation_arguments_for_record(arguments),
                    group,
                    stack_entry,
                )
            }
            None => (
                registry_operation
                    .as_ref()
                    .map(|descriptor| descriptor.path().to_string()),
                registry_operation
                    .as_ref()
                    .map(|descriptor| descriptor.display_name().to_string()),
                None,
                None,
                None,
            ),
        };

        let execution = match execute_event(&mut inner, &event) {
            Ok(outcome) => outcome,
            Err(error) => {
                inner.state.set_status_line(error.clone());
                let record = EditorEventRecord {
                    event_id,
                    sequence,
                    source,
                    event,
                    operation_id: operation_id.clone(),
                    operation_display_name: operation_display_name.clone(),
                    operation_arguments: operation_arguments.clone(),
                    operation_group: operation_group.clone(),
                    effects: vec![
                        EditorEventEffect::PresentationChanged,
                        EditorEventEffect::ReflectionChanged,
                    ],
                    undo_policy,
                    before_revision,
                    after_revision,
                    result: EditorEventResult::failure(error.clone()),
                };
                Self::refresh_reflection_locked(&mut inner);
                inner.journal.push(record.clone());
                inner.event_listeners.notify(&record);
                return Err(error);
            }
        };

        let record = EditorEventRecord {
            event_id,
            sequence,
            source,
            event,
            operation_id,
            operation_display_name,
            operation_arguments,
            operation_group,
            effects: execution.effects().to_vec(),
            undo_policy,
            before_revision,
            after_revision,
            result: EditorEventResult::success(event_result_value(
                after_revision,
                execution.changed(),
            )),
        };
        if let Some((operation_id, display_name, operation_group)) = explicit_stack_entry {
            inner.operation_stack.record(
                EditorOperationStackEntry::new(
                    operation_id,
                    display_name,
                    record.source.clone(),
                    record.sequence.0,
                )
                .with_operation_group(operation_group),
            );
        } else if execution.changed()
            && matches!(record.event, EditorEvent::WorkbenchMenu(MenuAction::Undo))
        {
            inner.operation_stack.move_undo_to_redo();
        } else if execution.changed()
            && matches!(record.event, EditorEvent::WorkbenchMenu(MenuAction::Redo))
        {
            inner.operation_stack.move_redo_to_undo();
        } else if let Some(descriptor) = registry_operation.as_ref() {
            if descriptor.undoable().is_some() && record.result.error.is_none() {
                inner.operation_stack.record(EditorOperationStackEntry::new(
                    descriptor.path().clone(),
                    descriptor.display_name().to_string(),
                    record.source.clone(),
                    record.sequence.0,
                ));
            }
        }
        Self::refresh_reflection_locked(&mut inner);
        inner.journal.push(record.clone());
        inner.event_listeners.notify(&record);
        Ok(record)
    }
}

fn dynamic_operation_for_event(
    inner: &crate::core::editor_event::runtime::editor_event_runtime_state::EditorEventRuntimeState,
    event: &EditorEvent,
) -> Option<EditorOperationDescriptor> {
    let path = match event {
        EditorEvent::Inspector(_) => "inspector.field.apply_batch",
        _ => return None,
    };
    let path = EditorOperationPath::parse(path).ok()?;
    inner.operation_registry.descriptor(&path).cloned()
}

fn operation_arguments_for_record(arguments: Value) -> Option<Value> {
    match arguments {
        Value::Null => None,
        Value::Array(values) if values.is_empty() => None,
        other => Some(other),
    }
}

impl EditorEventDispatcher for EditorEventRuntime {
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
        let event = normalize_editor_event_binding(&binding)?;
        self.dispatch_normalized_event(source, event)
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

impl EditorEventRuntime {
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
                self.invoke_operation(operation_source_for_event_source(source), invocation)
                    .map(Some)
            }
            EditorUiBindingPayload::EditorCommand { command_id } => self
                .dispatch_editor_command_binding(command_id, source)
                .map(Some),
            _ => Ok(None),
        }
    }

    fn dispatch_editor_command_binding(
        &self,
        command_id: &str,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        let registry = EditorCommandRegistry::default_workbench();
        let descriptor = registry.command(command_id).ok_or_else(|| {
            EditorCommandDispatchError::UnknownCommand(command_id.to_string()).to_string()
        })?;

        match descriptor.action() {
            EditorCommandAction::Menu(action) => {
                if let Some(operation_id) = operation_path_for_menu_action(action) {
                    return self.invoke_operation(
                        operation_source_for_event_source(source),
                        EditorOperationInvocation::new(operation_id),
                    );
                }
                self.dispatch_normalized_event(source, EditorEvent::WorkbenchMenu(action.clone()))
            }
            EditorCommandAction::Operation(operation_id) => self.invoke_operation(
                operation_source_for_event_source(source),
                EditorOperationInvocation::new(operation_id.clone()),
            ),
            EditorCommandAction::OpenCommandPalette => self.dispatch_normalized_event(
                source,
                EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
            ),
        }
    }

    fn record_material_component_lab_feedback(
        &self,
        source: EditorEventSource,
        binding: &EditorUiBinding,
    ) -> EditorEventRecord {
        let mut inner = self.lock_inner();
        inner.next_event_id += 1;
        inner.next_sequence += 1;

        let revision = inner.revision;
        let node_path = binding_node_path(binding);
        let event = EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path,
            pressed: false,
        });
        let undo_policy = undo_policy_for_event(&event);
        let record = EditorEventRecord {
            event_id: EditorEventId::new(inner.next_event_id),
            sequence: EditorEventSequence::new(inner.next_sequence),
            source,
            event,
            operation_id: None,
            operation_display_name: Some("Material Component Lab Feedback".to_string()),
            operation_arguments: None,
            operation_group: Some("MaterialComponentLab".to_string()),
            effects: Vec::new(),
            undo_policy,
            before_revision: revision,
            after_revision: revision,
            result: EditorEventResult::success(event_result_value(revision, false)),
        };
        inner.journal.push(record.clone());
        inner.event_listeners.notify(&record);
        record
    }

    fn record_component_lab_preview_action(
        &self,
        source: EditorEventSource,
        binding: &EditorUiBinding,
        action_id: &str,
    ) -> EditorEventRecord {
        let mut inner = self.lock_inner();
        inner.next_event_id += 1;
        inner.next_sequence += 1;

        let revision = inner.revision;
        let node_path = component_lab_preview_node_path(binding, action_id);
        let event = EditorEvent::Transient(EditorEventTransient::PressNode {
            node_path: node_path.clone(),
            pressed: false,
        });
        let undo_policy = undo_policy_for_event(&event);
        let record = EditorEventRecord {
            event_id: EditorEventId::new(inner.next_event_id),
            sequence: EditorEventSequence::new(inner.next_sequence),
            source,
            event,
            operation_id: None,
            operation_display_name: Some("Component Lab Preview Action".to_string()),
            operation_arguments: Some(serde_json::json!({
                "control_id": binding.path().control_id.clone(),
                "node_path": node_path,
                "action_id": action_id,
            })),
            operation_group: Some("ComponentLabPreview".to_string()),
            effects: Vec::new(),
            undo_policy,
            before_revision: revision,
            after_revision: revision,
            result: EditorEventResult::success(event_result_value(revision, false)),
        };
        inner.journal.push(record.clone());
        inner.event_listeners.notify(&record);
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
