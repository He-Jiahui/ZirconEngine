use crate::ui::EditorUiBinding;

use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventEnvelope, EditorEventId, EditorEventRecord,
    EditorEventResult, EditorEventSequence, EditorEventSource,
};

use super::binding_normalization::normalize_binding;
use super::editor_event_dispatcher::EditorEventDispatcher;
use super::editor_event_runtime::EditorEventRuntime;
use super::execution::{event_result_value, execute_event, undo_policy_for_event};

impl EditorEventRuntime {
    fn dispatch_normalized_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
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

        let execution = match execute_event(&mut inner, &event) {
            Ok(outcome) => outcome,
            Err(error) => {
                inner.state.set_status_line(error.clone());
                let record = EditorEventRecord {
                    event_id,
                    sequence,
                    source,
                    event,
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
                return Err(error);
            }
        };

        let record = EditorEventRecord {
            event_id,
            sequence,
            source,
            event,
            effects: execution.effects.clone(),
            undo_policy,
            before_revision,
            after_revision,
            result: EditorEventResult::success(event_result_value(
                after_revision,
                execution.changed,
            )),
        };
        Self::refresh_reflection_locked(&mut inner);
        inner.journal.push(record.clone());
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
        binding: EditorUiBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        let event = normalize_binding(&binding)?;
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
