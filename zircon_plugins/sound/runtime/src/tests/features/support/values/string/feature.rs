#[path = "../../../../string/feature/support_display_name.rs"]
mod display_name;
#[path = "../../../../support/values/string/feature/id.rs"]
mod id;
#[path = "../../../../string/feature/support_owner_plugin.rs"]
mod owner_plugin;

pub(in super::super::super) use display_name::feature_display_name_string_from_plugin_toml;
pub(in super::super::super) use id::feature_id_string_from_plugin_toml;
pub(in super::super::super) use owner_plugin::feature_owner_plugin_string_from_plugin_toml;
