#[path = "../../../support/values/packaging/defaults.rs"]
mod defaults;
#[path = "packaging/list.rs"]
mod list;
#[path = "packaging/raw.rs"]
mod raw;

pub(in super::super) use defaults::default_packaging_strategy_list_from_plugin_toml;
