use zircon_runtime_interface::ui::binding::{UiBindingValue, UiEventKind};

use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::{BuiltinAssetSurfaceTemplateBridge, common::dispatch_editor_binding};

pub(crate) fn dispatch_builtin_asset_surface_control(
    runtime: &EditorHostEventController,
    bridge: &BuiltinAssetSurfaceTemplateBridge,
    control_id: &str,
    event_kind: UiEventKind,
    arguments: Vec<UiBindingValue>,
) -> Option<Result<UiHostEventEffects, String>> {
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
