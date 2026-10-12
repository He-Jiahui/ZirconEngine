use crate::plugin::RuntimeExtensionRegistry;

use super::super::contributions::push_runtime_extension_result;

pub(super) fn merge_runtime_provider_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for provider in extensions.virtual_geometry_runtime_providers() {
        push_runtime_extension_result(
            registry.register_virtual_geometry_runtime_provider(provider.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for provider in extensions.hybrid_gi_runtime_providers() {
        push_runtime_extension_result(
            registry.register_hybrid_gi_runtime_provider(provider.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for provider in extensions.solari_runtime_providers() {
        push_runtime_extension_result(
            registry.register_solari_runtime_provider(provider.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
}
