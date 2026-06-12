mod capabilities;
mod identity;
mod systems;
mod target_modes;

use crate::{plugin::PluginModuleManifest, RuntimeTargetMode};

use self::{
    capabilities::validate_runtime_plugin_package_module_capabilities,
    systems::validate_runtime_plugin_package_module_system_contracts,
    target_modes::validate_runtime_plugin_package_module_target_modes,
};

pub(super) fn validate_runtime_plugin_package_module_row<'a>(
    package_id: &str,
    package_supported_targets: &'a [RuntimeTargetMode],
    module: &'a PluginModuleManifest,
    seen_names: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    identity::validate_runtime_plugin_package_module_identity(
        package_id,
        module,
        seen_names,
        diagnostics,
    );
    validate_runtime_plugin_package_module_capabilities(module, diagnostics);
    validate_runtime_plugin_package_module_system_contracts(package_id, module, diagnostics);
    validate_runtime_plugin_package_module_target_modes(
        package_supported_targets,
        module,
        diagnostics,
    );
}
