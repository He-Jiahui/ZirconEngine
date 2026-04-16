use zircon_editor_ui::EditorUiBinding;

use crate::editor_event::{EditorEvent, EditorEventEnvelope, EditorEventRecord, EditorEventSource};

pub trait EditorEventDispatcher {
    fn dispatch_envelope(&self, envelope: EditorEventEnvelope)
        -> Result<EditorEventRecord, String>;
    fn dispatch_binding(
        &self,
        binding: EditorUiBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String>;
    fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String>;
}
