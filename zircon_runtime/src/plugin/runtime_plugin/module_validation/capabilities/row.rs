mod kind_prefix;
mod uniqueness;

use crate::plugin::PluginModuleManifest;

use self::{
    kind_prefix::validate_runtime_plugin_module_capability_kind_prefix,
    uniqueness::validate_runtime_plugin_module_capability_uniqueness,
};

/// 保留包与 feature 各自的诊断契约：包的命名空间检查已包含字段检查，feature 需另行提供。
/// 一项规则失败仍继续报告其余约束，便于注册报告一次呈现同一声明的全部问题。
pub(super) fn validate_runtime_plugin_module_capability_row(
    manifest_label: &str,
    module: &PluginModuleManifest,
    capability: &str,
    is_duplicate: bool,
    validate_field: Option<fn(&str, &str, &mut Vec<String>)>,
    validate_namespace: fn(&str, &str, &mut Vec<String>),
    diagnostics: &mut Vec<String>,
) {
    if let Some(validate_field) = validate_field {
        validate_field("module capability", capability, diagnostics);
    }
    validate_namespace("module capability", capability, diagnostics);
    validate_runtime_plugin_module_capability_kind_prefix(
        manifest_label,
        module,
        capability,
        diagnostics,
    );
    validate_runtime_plugin_module_capability_uniqueness(
        manifest_label,
        module,
        capability,
        is_duplicate,
        diagnostics,
    );
}
