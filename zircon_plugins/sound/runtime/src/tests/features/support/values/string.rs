#[path = "../../../support/values/string/dependency.rs"]
mod dependency;
#[path = "string/feature.rs"]
mod feature;
#[path = "string/module.rs"]
mod module;
#[path = "string/raw.rs"]
mod raw;

pub(in super::super) use dependency::{
    dependency_capability_string_from_plugin_toml, dependency_plugin_id_string_from_plugin_toml,
};
pub(in super::super) use feature::{
    feature_display_name_string_from_plugin_toml, feature_id_string_from_plugin_toml,
    feature_owner_plugin_string_from_plugin_toml,
};
pub(in super::super) use module::{
    module_crate_name_string_from_plugin_toml, module_name_string_from_plugin_toml,
};
