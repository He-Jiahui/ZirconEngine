#[path = "../../../support/values/boolean/dependency.rs"]
mod dependency;
#[path = "../../../support/values/boolean/feature.rs"]
mod feature;
#[path = "boolean/raw.rs"]
mod raw;

pub(in super::super) use dependency::dependency_primary_bool_from_plugin_toml;
pub(in super::super) use feature::enabled_by_default_bool_from_plugin_toml;
