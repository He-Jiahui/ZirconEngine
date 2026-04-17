use zircon_editor_ui::EditorUiBinding;

use crate::editor_event::{EditorEventEnvelope, EditorEventRuntime, EditorEventSource};
use crate::host::slint_host::event_bridge::{SlintDispatchEffects, apply_record_effects};

pub(crate) fn dispatch_envelope(
    runtime: &EditorEventRuntime,
    envelope: EditorEventEnvelope,
) -> Result<SlintDispatchEffects, String> {
    let record = runtime.dispatch_envelope(envelope)?;
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(&mut effects, &record);
    Ok(effects)
}

pub(crate) fn dispatch_editor_binding(
    runtime: &EditorEventRuntime,
    binding: EditorUiBinding,
) -> Result<SlintDispatchEffects, String> {
    let record = runtime.dispatch_binding(binding, EditorEventSource::Slint)?;
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(&mut effects, &record);
    Ok(effects)
}
