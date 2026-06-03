mod capabilities;
mod identity;
mod target_modes;

use crate::plugin::{PluginFeatureBundleManifest, PluginModuleManifest};

use self::{
    capabilities::validate_runtime_plugin_feature_module_capabilities,
    identity::validate_runtime_plugin_feature_module_identity,
    target_modes::validate_runtime_plugin_feature_module_target_modes,
};

pub(super) fn validate_runtime_plugin_feature_module_row<'a>(
    feature: &PluginFeatureBundleManifest,
    module: &'a PluginModuleManifest,
    seen_names: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_module_identity(feature, module, seen_names, diagnostics);
    validate_runtime_plugin_feature_module_capabilities(module, diagnostics);
    validate_runtime_plugin_feature_module_target_modes(module, diagnostics);
}
