use crate::core::editor_event::{
    EditorEvent, EditorEventDispatcher, EditorEventEnvelope, EditorEventJournal, EditorEventRecord,
    EditorEventSource,
};
use crate::ui::host::EditorHostEventController;

impl EditorHostEventController {
    pub fn journal(&self) -> EditorEventJournal {
        self.context().events().journal()
    }

    pub fn dispatch_envelope(
        &self,
        envelope: EditorEventEnvelope,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_envelope(self, envelope)
    }

    pub fn dispatch_binding(
        &self,
        binding: crate::ui::binding::EditorUiBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_binding(self, binding.as_ui_binding(), source)
    }

    pub fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String> {
        <Self as EditorEventDispatcher>::dispatch_event(self, source, event)
    }
}
