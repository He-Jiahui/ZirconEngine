//! 包、模块、能力和接口共享这一组声明文本规则；各调用方仍须单独验证所有权与跨行关系。
#[path = "shape/field.rs"]
mod field;
#[path = "shape/identity.rs"]
mod identity;
#[path = "shape/namespace.rs"]
mod namespace;
#[path = "shape/token.rs"]
mod token;

pub(in crate::plugin::runtime_plugin) use field::validate_runtime_plugin_package_field;
pub(in crate::plugin::runtime_plugin) use identity::validate_runtime_plugin_package_id;
pub(in crate::plugin::runtime_plugin) use namespace::validate_runtime_plugin_package_namespace;
pub(in crate::plugin::runtime_plugin) use token::{
    is_lowercase_runtime_plugin_token, validate_runtime_plugin_package_token,
};
