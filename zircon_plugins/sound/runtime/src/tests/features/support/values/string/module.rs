#[path = "../../../../string/module/support_crate_name.rs"]
mod crate_name;
#[path = "../../../../support/values/string/module/name.rs"]
mod name;

pub(in super::super::super) use crate_name::module_crate_name_string_from_plugin_toml;
pub(in super::super::super) use name::module_name_string_from_plugin_toml;
