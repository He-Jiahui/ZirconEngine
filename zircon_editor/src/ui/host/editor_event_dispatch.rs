use crate::core::editor_event::{
    EditorEvent, EditorEventDispatcher, EditorEventEffect, EditorEventEnvelope, EditorEventId,
    EditorEventRecord, EditorEventResult, EditorEventRuntime, EditorEventSequence,
    EditorEventSource,
};
use crate::core::editor_operation::{EditorOperationPath, EditorOperationStackEntry};
use crate::ui::binding::EditorUiBinding;
use crate::ui::binding_dispatch::editor_event_normalization::normalize_editor_event_binding;
use zircon_runtime::ui::binding::UiEventBinding;

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
        operation: Option<(EditorOperationPath, String, bool)>,
    ) -> Result<EditorEventRecord, String> {
        let mut inner = self.inner.lock().unwrap();
        inner.next_event_id += 1;
        inner.next_sequence += 1;

        let before_revision = inner.revision;
        let after_revision = before_revision + 1;
        inner.revision = after_revision;

        let event_id = EditorEventId::new(inner.next_event_id);
        let sequence = EditorEventSequence::new(inner.next_sequence);
        let undo_policy = undo_policy_for_event(&event);
        let registry_operation = operation.is_none().then(|| {
            inner
                .operation_registry
                .descriptor_for_event(&event)
                .cloned()
        });
        let registry_operation = registry_operation.flatten();
        let (operation_id, operation_display_name, explicit_stack_entry) = match operation {
            Some((operation_id, operation_display_name, undoable)) => {
                let stack_entry =
                    undoable.then(|| (operation_id.clone(), operation_display_name.clone()));
                (
                    Some(operation_id.to_string()),
                    Some(operation_display_name),
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
            effects: execution.effects().to_vec(),
            undo_policy,
            before_revision,
            after_revision,
            result: EditorEventResult::success(event_result_value(
                after_revision,
                execution.changed(),
            )),
        };
        if let Some((operation_id, display_name)) = explicit_stack_entry {
            inner.operation_stack.record(EditorOperationStackEntry::new(
                operation_id,
                display_name,
                record.sequence.0,
            ));
        } else if let Some(descriptor) = registry_operation.as_ref() {
            if descriptor.undoable().is_some() && record.result.error.is_none() {
                inner.operation_stack.record(EditorOperationStackEntry::new(
                    descriptor.path().clone(),
                    descriptor.display_name().to_string(),
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
