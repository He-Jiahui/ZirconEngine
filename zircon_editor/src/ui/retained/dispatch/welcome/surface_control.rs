use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventKind};

use crate::ui::binding_dispatch::{dispatch_welcome_binding, WelcomeHostEvent};

use super::super::BuiltinWelcomeSurfaceTemplateBridge;

pub(crate) fn dispatch_builtin_welcome_surface_control(
    bridge: &BuiltinWelcomeSurfaceTemplateBridge,
    control_id: &str,
    event_kind: UiEventKind,
    arguments: Vec<UiBindingValue>,
) -> Option<Result<WelcomeHostEvent, String>> {
    let binding = match bridge.binding_for_control(control_id, event_kind) {
        Some(binding) if arguments.is_empty() => Ok(binding.clone()),
        Some(binding) => binding
            .with_arguments(arguments)
            .map_err(|error| error.to_string()),
        None => return None,
    };

    Some(match binding {
        Ok(binding) => dispatch_welcome_binding(&binding).map_err(|error| error.to_string()),
        Err(error) => Err(error),
    })
}
