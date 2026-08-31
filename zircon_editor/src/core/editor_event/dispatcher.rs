use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventRecord, EditorEventSource,
};
use zircon_runtime_interface::ui::binding::UiEventBinding;

pub trait EditorEventDispatcher {
    type Error: std::error::Error + 'static;

    fn dispatch_envelope(
        &self,
        envelope: EditorEventEnvelope,
    ) -> Result<EditorEventRecord, Self::Error>;

    fn dispatch_binding(
        &self,
        binding: UiEventBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, Self::Error>;

    fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, Self::Error>;
}
