use crate::plugin::PluginModuleManifest;

use super::row::validate_runtime_plugin_module_capability_row;

/// 去重回调使用调用端预先建立的序号投影；验证期间不可过滤或重排能力列表。
/// 重复范围是当前模块，多个模块声明相同能力并不由此判为冲突。
pub(super) fn validate_runtime_plugin_module_capability_rows(
    manifest_label: &str,
    module: &PluginModuleManifest,
    validate_field: Option<fn(&str, &str, &mut Vec<String>)>,
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    is_duplicate: impl Fn(usize) -> bool,
    diagnostics: &mut Vec<String>,
) {
    for (index, capability) in module.capabilities.iter().enumerate() {
        validate_runtime_plugin_module_capability_row(
            manifest_label,
            module,
            capability,
            is_duplicate(index),
            validate_field,
            validate_namespace,
            diagnostics,
        );
    }
}
