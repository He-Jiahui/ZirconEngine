use crate::plugin::RuntimeExtensionRegistry;

use super::super::contributions::push_runtime_extension_result;

pub(super) fn merge_render_feature_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for render_feature in extensions.render_features() {
        push_runtime_extension_result(
            registry.register_render_feature(render_feature.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for executor in extensions.render_pass_executors() {
        push_runtime_extension_result(
            registry.register_render_pass_executor(executor.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
}
