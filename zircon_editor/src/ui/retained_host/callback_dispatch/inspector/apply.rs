use crate::core::editor_event::{
    EditorEventEnvelope, EditorEventRuntime, EditorEventSource, EditorInspectorEvent,
};
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_envelope;

#[cfg(test)]
pub(crate) fn dispatch_inspector_apply(
    runtime: &EditorEventRuntime,
    event: EditorInspectorEvent,
) -> Result<UiHostEventEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::RetainedHost,
            crate::core::editor_event::EditorEvent::Inspector(event),
        ),
    )
}
