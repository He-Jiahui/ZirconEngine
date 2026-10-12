use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventKind};

use crate::core::editor_event::ViewInstanceId;
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::{
    common::{dispatch::dispatch_editor_binding_for_view, dispatch_editor_binding},
    BuiltinViewportToolbarTemplateBridge,
};

pub(crate) fn dispatch_builtin_viewport_toolbar_control(
    runtime: &EditorHostEventController,
    bridge: &BuiltinViewportToolbarTemplateBridge,
    control_id: &str,
    event_kind: UiEventKind,
    arguments: Vec<UiBindingValue>,
    view_id: Option<&ViewInstanceId>,
) -> Option<Result<UiHostEventEffects, String>> {
    let binding = match bridge.binding_for_control(control_id, event_kind) {
        Some(binding) if arguments.is_empty() => Ok(binding.clone()),
        Some(binding) => binding
            .with_arguments(arguments)
            .map_err(|error| error.to_string()),
        None => return None,
    };

    Some(match binding {
        Ok(binding) => match view_id {
            Some(view_id) => dispatch_editor_binding_for_view(runtime, binding, view_id.clone()),
            None => dispatch_editor_binding(runtime, binding),
        },
        Err(error) => Err(error),
    })
}
