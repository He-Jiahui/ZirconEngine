use crate::core::editor_event::{EditorEventEnvelope, EditorEventRuntime, EditorEventSource};
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;
use crate::ui::workbench::event::core_layout_command_from_ui;
use crate::ui::workbench::layout::LayoutCommand;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_layout_command(
    runtime: &EditorEventRuntime,
    command: LayoutCommand,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::Slint,
            crate::core::editor_event::EditorEvent::Layout(core_layout_command_from_ui(command)),
        ),
    )
}
