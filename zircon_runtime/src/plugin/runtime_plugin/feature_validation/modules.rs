mod row;
mod rows;

use crate::plugin::PluginFeatureBundleManifest;

pub(super) fn validate_runtime_plugin_feature_modules(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_feature_module_rows(feature, diagnostics);
}
