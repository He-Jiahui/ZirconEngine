#[path = "../../../support/values/module_kind/module.rs"]
mod module;
#[path = "../../../values/module_kind/features_raw.rs"]
mod raw;

pub(in super::super) use module::module_kind_value_from_plugin_toml;
