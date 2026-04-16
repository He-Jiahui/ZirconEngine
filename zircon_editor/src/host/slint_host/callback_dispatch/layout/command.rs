use crate::editor_event::{EditorEventEnvelope, EditorEventRuntime, EditorEventSource};
use crate::host::slint_host::event_bridge::SlintDispatchEffects;
use crate::LayoutCommand;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_layout_command(
    runtime: &EditorEventRuntime,
    command: LayoutCommand,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::Slint,
            crate::EditorEvent::Layout(command),
        ),
    )
}
