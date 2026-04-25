use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventRuntime, EditorEventSource, LayoutCommand,
};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;
use crate::ui::workbench::event::{dispatch_editor_host_binding, EditorHostEvent};

use super::super::common::dispatch_envelope;
use super::control::dispatch_builtin_host_menu_action;

pub(crate) fn dispatch_menu_action(
    runtime: &EditorEventRuntime,
    action: &str,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, slint_menu_action(action)?)
}

pub(crate) fn dispatch_host_menu_action_with_template_fallback(
    runtime: &EditorEventRuntime,
    bridge: &super::super::BuiltinHostWindowTemplateBridge,
    action: &str,
) -> Result<SlintDispatchEffects, String> {
    if let Some(result) = dispatch_builtin_host_menu_action(runtime, bridge, action) {
        return result;
    }
    dispatch_menu_action(runtime, action)
}

pub(crate) fn slint_menu_action(action_id: &str) -> Result<EditorEventEnvelope, String> {
    if let Some(name) = action_id.strip_prefix("SavePreset.") {
        let name = if name.is_empty() { "current" } else { name };
        return Ok(EditorEventEnvelope::new(
            EditorEventSource::Slint,
            EditorEvent::Layout(LayoutCommand::SavePreset {
                name: name.to_string(),
            }),
        ));
    }

    if let Some(name) = action_id.strip_prefix("LoadPreset.") {
        let name = if name.is_empty() { "current" } else { name };
        return Ok(EditorEventEnvelope::new(
            EditorEventSource::Slint,
            EditorEvent::Layout(LayoutCommand::LoadPreset {
                name: name.to_string(),
            }),
        ));
    }

    let binding = EditorUiBinding::new(
        "WorkbenchMenuBar",
        action_id,
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action(action_id),
    );
    let event = dispatch_editor_host_binding(&binding).map_err(|error| error.to_string())?;
    let EditorHostEvent::Menu(action) = event;
    Ok(EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::WorkbenchMenu(action),
    ))
}
