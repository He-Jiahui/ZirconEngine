mod capabilities;
mod identity;
mod systems;
mod target_modes;

use super::super::projection::RuntimePluginPackageValidationProjection;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginModuleManifest;

use self::{
    capabilities::validate_runtime_plugin_package_module_capabilities,
    systems::validate_runtime_plugin_package_module_system_contracts,
    target_modes::validate_runtime_plugin_package_module_target_modes,
};

pub(super) fn validate_runtime_plugin_package_module_row<'a>(
    package_id: &str,
    package_supported_targets: &'a [RuntimeTargetMode],
    module: &'a PluginModuleManifest,
    module_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    identity::validate_runtime_plugin_package_module_identity(
        package_id,
        module,
        projection.package_module_name_is_duplicate(module_index),
        diagnostics,
    );
    validate_runtime_plugin_package_module_capabilities(
        module,
        module_index,
        projection,
        diagnostics,
    );
    validate_runtime_plugin_package_module_system_contracts(
        package_id,
        module,
        module_index,
        projection,
        diagnostics,
    );
    validate_runtime_plugin_package_module_target_modes(
        package_supported_targets,
        module,
        diagnostics,
    );
}
