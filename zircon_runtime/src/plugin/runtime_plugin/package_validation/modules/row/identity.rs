mod crate_name;
mod name;

use crate::plugin::PluginModuleManifest;

use self::{
    crate_name::validate_runtime_plugin_package_module_crate_name,
    name::validate_runtime_plugin_package_module_name,
};

pub(super) fn validate_runtime_plugin_package_module_identity<'a>(
    package_id: &str,
    module: &'a PluginModuleManifest,
    seen_names: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_module_name(package_id, module, seen_names, diagnostics);
    validate_runtime_plugin_package_module_crate_name(&module.crate_name, diagnostics);
}
