use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

use super::super::super::super::module_validation::validate_runtime_plugin_module_target_modes;

pub(super) fn validate_runtime_plugin_package_module_target_modes(
    package_supported_targets: &[RuntimeTargetMode],
    module: &PluginModuleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_target_modes(
        "runtime plugin package manifest",
        module,
        Some(("package supported_targets", package_supported_targets)),
        diagnostics,
    );
}
