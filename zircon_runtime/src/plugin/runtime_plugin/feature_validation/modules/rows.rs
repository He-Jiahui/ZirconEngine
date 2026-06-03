mod state;

use crate::plugin::PluginFeatureBundleManifest;

use self::state::new_runtime_plugin_feature_module_row_state;
use super::row::validate_runtime_plugin_feature_module_row;

pub(super) fn validate_runtime_plugin_feature_module_rows(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen_names = new_runtime_plugin_feature_module_row_state();
    for module in &feature.modules {
        validate_runtime_plugin_feature_module_row(feature, module, &mut seen_names, diagnostics);
    }
}
