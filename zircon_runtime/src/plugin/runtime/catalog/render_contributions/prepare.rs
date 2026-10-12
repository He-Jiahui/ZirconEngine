use crate::plugin::RuntimeExtensionRegistry;

use super::super::contributions::push_runtime_extension_result;

pub(super) fn merge_runtime_prepare_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for collector in extensions.runtime_prepare_collectors() {
        push_runtime_extension_result(
            registry.register_runtime_prepare_collector(collector.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
}
