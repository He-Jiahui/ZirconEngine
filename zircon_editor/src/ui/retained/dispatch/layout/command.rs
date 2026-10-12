use crate::core::editor_event::{EditorEventEnvelope, EditorEventSource};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::workbench::event::core_layout_command_from_ui;
use crate::ui::workbench::layout::LayoutCommand;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_layout_command(
    runtime: &EditorHostEventController,
    command: LayoutCommand,
) -> Result<UiHostEventEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::RetainedHost,
            crate::core::editor_event::EditorEvent::Layout(core_layout_command_from_ui(command)),
        ),
    )
}
