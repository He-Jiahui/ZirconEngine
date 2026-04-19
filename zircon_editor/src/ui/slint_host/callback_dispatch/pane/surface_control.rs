use zircon_ui::binding::UiBindingValue;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

use super::super::{common::dispatch_editor_binding, BuiltinPaneSurfaceTemplateBridge};

pub(crate) fn dispatch_builtin_pane_surface_control(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinPaneSurfaceTemplateBridge,
    control_id: &str,
    event_kind: zircon_ui::binding::UiEventKind,
    arguments: Vec<UiBindingValue>,
) -> Option<Result<SlintDispatchEffects, String>> {
    let binding = match bridge.binding_for_control(control_id, event_kind) {
        Some(binding) if arguments.is_empty() => Ok(binding.clone()),
        Some(binding) => binding
            .with_arguments(arguments)
            .map_err(|error| error.to_string()),
        None => return None,
    };

    Some(match binding {
        Ok(binding) => dispatch_editor_binding(runtime, binding),
        Err(error) => Err(error),
    })
}
