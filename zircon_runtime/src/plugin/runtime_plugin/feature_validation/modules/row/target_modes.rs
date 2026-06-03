use crate::plugin::PluginModuleManifest;

use super::super::super::super::module_validation::validate_runtime_plugin_module_target_modes;

pub(super) fn validate_runtime_plugin_feature_module_target_modes(
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_target_modes(
        "runtime plugin feature manifest",
        module,
        None,
        diagnostics,
    );
}
