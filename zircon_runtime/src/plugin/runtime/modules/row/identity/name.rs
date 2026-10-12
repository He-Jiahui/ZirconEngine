use crate::plugin::{PluginFeatureBundleManifest, PluginModuleManifest};

use super::super::super::super::super::module_validation::validate_runtime_plugin_module_name;
use super::super::super::super::shape::{
    validate_runtime_plugin_feature_field, validate_runtime_plugin_feature_namespace,
};

// 功能模块归属整个功能 ID 的命名空间；只归属插件的名字不足以定位具体功能。
pub(super) fn validate_runtime_plugin_feature_module_name(
    feature: &PluginFeatureBundleManifest,
    module: &PluginModuleManifest,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_name(
        "runtime plugin feature manifest",
        "feature id",
        &feature.id,
        module,
        is_duplicate,
        validate_runtime_plugin_feature_field,
        validate_runtime_plugin_feature_namespace,
        diagnostics,
    );
}
