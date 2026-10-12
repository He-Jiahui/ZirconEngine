//! 静态清单测试共用的值转换表；输入范围由仓内插件文件及其测试投影限定。
#[path = "values/array.rs"]
mod array;
#[path = "values/boolean.rs"]
mod boolean;
#[path = "../../support/values/capability_status.rs"]
mod capability_status;
#[path = "values/maturity.rs"]
mod maturity;
#[path = "values/module_kind.rs"]
mod module_kind;
#[path = "values/target_mode.rs"]
mod target_mode;

pub(super) use array::string_array_values;
pub(super) use boolean::bool_from_plugin_toml;
pub(super) use capability_status::capability_status_from_plugin_toml;
pub(super) use maturity::maturity_from_plugin_toml;
pub(super) use module_kind::plugin_module_kind_from_plugin_toml;
pub(super) use target_mode::runtime_target_mode_from_plugin_toml;
