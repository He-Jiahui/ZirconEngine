use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventRecord, EditorEventSource,
};
use zircon_runtime_interface::ui::binding::UiEventBinding;

pub trait EditorEventDispatcher {
    fn dispatch_envelope(&self, envelope: EditorEventEnvelope)
        -> Result<EditorEventRecord, String>;
    fn dispatch_binding(
        &self,
        binding: UiEventBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, String>;
    fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, String>;
}
