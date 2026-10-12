#[path = "identity/crate_name.rs"]
mod crate_name;
#[path = "identity/name.rs"]
mod name;

use crate::plugin::PluginModuleManifest;

use self::{
    crate_name::validate_runtime_plugin_package_module_crate_name,
    name::validate_runtime_plugin_package_module_name,
};

pub(super) fn validate_runtime_plugin_package_module_identity(
    package_id: &str,
    module: &PluginModuleManifest,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_module_name(package_id, module, is_duplicate, diagnostics);
    validate_runtime_plugin_package_module_crate_name(&module.crate_name, diagnostics);
}
