use super::super::super::projection::RuntimePluginPackageValidationProjection;
use crate::plugin::PluginModuleManifest;

use super::super::super::super::module_validation::validate_runtime_plugin_module_capabilities;
use super::super::super::validate_runtime_plugin_package_namespace;

// 将包投影中的逐项重复标记交给通用模块校验，避免包入口重新扫描或产生不同诊断语义。
pub(super) fn validate_runtime_plugin_package_module_capabilities(
    module: &PluginModuleManifest,
    module_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_capabilities(
        "runtime plugin package manifest",
        module,
        None,
        validate_runtime_plugin_package_namespace,
        |capability_index| {
            projection.package_module_capability_is_duplicate(module_index, capability_index)
        },
        diagnostics,
    );
}
