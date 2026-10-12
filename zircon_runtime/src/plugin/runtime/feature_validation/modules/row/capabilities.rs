use super::super::super::projection::RuntimePluginFeatureValidationProjection;
use crate::plugin::PluginModuleManifest;

use super::super::super::super::module_validation::validate_runtime_plugin_module_capabilities;
use super::super::super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_namespace,
};

// 复用包模块的种类前缀等规则，但传入功能专属诊断与重复投影，
// 让独立功能和包内嵌功能保持相同的模块能力约束。
pub(super) fn validate_runtime_plugin_feature_module_capabilities(
    module: &PluginModuleManifest,
    module_index: usize,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_capabilities(
        "runtime plugin feature manifest",
        module,
        Some(validate_runtime_plugin_feature_field),
        validate_runtime_plugin_feature_namespace,
        |capability_index| {
            projection.module_capability_is_duplicate(module_index, capability_index)
        },
        diagnostics,
    );
}
