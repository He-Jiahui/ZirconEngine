use zircon_ui::UiBindingValue;

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::event_bridge::SlintDispatchEffects;

use super::super::{common::dispatch_editor_binding, BuiltinAssetSurfaceTemplateBridge};

pub(crate) fn dispatch_builtin_asset_surface_control(
    runtime: &EditorEventRuntime,
    bridge: &BuiltinAssetSurfaceTemplateBridge,
    control_id: &str,
    event_kind: zircon_ui::UiEventKind,
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
