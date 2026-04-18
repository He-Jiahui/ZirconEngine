use crate::core::editor_event::{
    EditorEventEnvelope, EditorEventRuntime, EditorEventSource, EditorInspectorEvent,
};
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

use super::super::common::dispatch_envelope;

#[cfg(test)]
pub(crate) fn dispatch_inspector_apply(
    runtime: &EditorEventRuntime,
    event: EditorInspectorEvent,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::Slint,
            crate::EditorEvent::Inspector(event),
        ),
    )
}
