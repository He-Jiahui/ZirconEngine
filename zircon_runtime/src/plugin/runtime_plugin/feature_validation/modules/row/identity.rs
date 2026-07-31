mod crate_name;
mod name;

use crate::plugin::{PluginFeatureBundleManifest, PluginModuleManifest};

use self::{
    crate_name::validate_runtime_plugin_feature_module_crate_name,
    name::validate_runtime_plugin_feature_module_name,
};

pub(super) fn validate_runtime_plugin_feature_module_identity(
    feature: &PluginFeatureBundleManifest,
    module: &PluginModuleManifest,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_module_name(feature, module, is_duplicate, diagnostics);
    validate_runtime_plugin_feature_module_crate_name(&module.crate_name, diagnostics);
}
