use crate::plugin::RuntimeExtensionRegistry;

use super::super::contributions::push_runtime_extension_result;

pub(super) fn merge_component_descriptor_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for component in extensions.components() {
        push_runtime_extension_result(
            registry.register_component(component.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    #[cfg(feature = "ui")]
    for ui_component in extensions.ui_components() {
        push_runtime_extension_result(
            registry.register_ui_component(ui_component.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
}
