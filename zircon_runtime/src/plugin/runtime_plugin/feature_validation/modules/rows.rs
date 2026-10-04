use crate::plugin::PluginFeatureBundleManifest;

use super::super::projection::RuntimePluginFeatureValidationProjection;
use super::row::validate_runtime_plugin_feature_module_row;

// 模块行号同时定位模块名和该模块内部的能力重复信息，遍历次序不得偏离投影源。
pub(super) fn validate_runtime_plugin_feature_module_rows(
    feature: &PluginFeatureBundleManifest,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    for (module_index, module) in feature.modules.iter().enumerate() {
        validate_runtime_plugin_feature_module_row(
            feature,
            module,
            module_index,
            projection,
            diagnostics,
        );
    }
}
