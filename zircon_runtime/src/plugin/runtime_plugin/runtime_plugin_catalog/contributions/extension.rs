use crate::plugin::RuntimeExtensionRegistry;

use super::super::descriptor_contributions::merge_descriptor_extension_registry_contributions;
use super::super::render_contributions::merge_render_extension_registry_contributions;
use super::diagnostic::push_runtime_extension_result;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn merge_extension_registry_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for module in extensions.modules() {
        push_runtime_extension_result(
            registry.register_module(module.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    merge_render_extension_registry_contributions(
        extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
    merge_descriptor_extension_registry_contributions(
        extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}
