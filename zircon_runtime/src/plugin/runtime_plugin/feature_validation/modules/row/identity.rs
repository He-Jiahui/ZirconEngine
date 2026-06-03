mod crate_name;
mod name;

use crate::plugin::{PluginFeatureBundleManifest, PluginModuleManifest};

use self::{
    crate_name::validate_runtime_plugin_feature_module_crate_name,
    name::validate_runtime_plugin_feature_module_name,
};

pub(super) fn validate_runtime_plugin_feature_module_identity<'a>(
    feature: &PluginFeatureBundleManifest,
    module: &'a PluginModuleManifest,
    seen_names: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_module_name(feature, module, seen_names, diagnostics);
    validate_runtime_plugin_feature_module_crate_name(&module.crate_name, diagnostics);
}
