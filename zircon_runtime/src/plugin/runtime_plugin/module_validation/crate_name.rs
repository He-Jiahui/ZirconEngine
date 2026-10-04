mod shape;
mod token;
mod underscore;

use self::{
    shape::validate_runtime_plugin_module_crate_name_shape,
    token::validate_runtime_plugin_module_crate_name_token,
    underscore::validate_runtime_plugin_module_crate_name_underscore,
};

/// 在包和 feature 的注册报告中校验供项目选择及导出计划消费的 crate 标识。
/// 必须通过完整组合规则才算名称有效；这里只检查声明形态，不确认 Cargo 包或动态产物存在。
pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_module_crate_name(
    manifest_label: &str,
    validate_field: fn(&str, &str, &mut Vec<String>),
    crate_name: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_module_crate_name_shape(validate_field, crate_name, diagnostics);
    validate_runtime_plugin_module_crate_name_token(manifest_label, crate_name, diagnostics);
    validate_runtime_plugin_module_crate_name_underscore(manifest_label, crate_name, diagnostics);
}
