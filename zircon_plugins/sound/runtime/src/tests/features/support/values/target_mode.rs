#[path = "../../../support/values/target_mode/list.rs"]
mod list;
#[path = "../../../support/values/target_mode/module.rs"]
mod module;
#[path = "../../../values/target_mode/features_raw.rs"]
mod raw;

pub(in super::super) use module::module_target_mode_list_from_plugin_toml;
