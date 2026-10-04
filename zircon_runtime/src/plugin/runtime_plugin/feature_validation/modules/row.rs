mod capabilities;
mod identity;
mod target_modes;

use super::super::projection::RuntimePluginFeatureValidationProjection;
use crate::plugin::{PluginFeatureBundleManifest, PluginModuleManifest};

use self::{
    capabilities::validate_runtime_plugin_feature_module_capabilities,
    identity::validate_runtime_plugin_feature_module_identity,
    target_modes::validate_runtime_plugin_feature_module_target_modes,
};

// 重复身份来自整份 feature 投影；每行按身份、能力、目标顺序累积诊断。
pub(super) fn validate_runtime_plugin_feature_module_row(
    feature: &PluginFeatureBundleManifest,
    module: &PluginModuleManifest,
    module_index: usize,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_module_identity(
        feature,
        module,
        projection.module_name_is_duplicate(module_index),
        diagnostics,
    );
    validate_runtime_plugin_feature_module_capabilities(
        module,
        module_index,
        projection,
        diagnostics,
    );
    validate_runtime_plugin_feature_module_target_modes(module, diagnostics);
}
