use crate::core::editor_event::{EditorEventEnvelope, EditorEventSource, EditorInspectorEvent};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_envelope;

#[cfg(test)]
pub(crate) fn dispatch_inspector_apply(
    runtime: &EditorHostEventController,
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
