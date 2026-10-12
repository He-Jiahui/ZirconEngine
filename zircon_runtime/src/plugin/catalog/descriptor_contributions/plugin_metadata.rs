use crate::plugin::RuntimeExtensionRegistry;

use super::super::contributions::push_runtime_extension_result;

pub(super) fn merge_plugin_metadata_descriptor_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for option in extensions.plugin_options() {
        push_runtime_extension_result(
            registry.register_plugin_option(option.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for event_catalog in extensions.plugin_event_catalogs() {
        push_runtime_extension_result(
            registry.register_plugin_event_catalog(event_catalog.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
}
