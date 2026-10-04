//! 功能清单共用的格式约束，不修改清单值，只向注册报告累积诊断。
//! 字段文本、插件标记与点分命名空间分开，避免对显示名施加机器标识符规则。

mod field;
mod namespace;
mod token;

pub(super) use field::validate_runtime_plugin_feature_field;
pub(super) use namespace::validate_runtime_plugin_feature_namespace;
pub(super) use token::validate_runtime_plugin_feature_token;
