#[path = "array/feature.rs"]
mod feature;
#[path = "array/list.rs"]
mod list;
#[path = "array/module.rs"]
mod module;
#[path = "array/raw.rs"]
mod raw;

pub(in super::super) use feature::feature_capability_list_from_plugin_toml;
pub(in super::super) use list::string_list_from_plugin_toml;
pub(in super::super) use module::module_capability_list_from_plugin_toml;
