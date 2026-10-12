#[path = "../../../support/runtime/identity/display_name.rs"]
mod display_name;
#[path = "identity/id.rs"]
mod id;
#[path = "../../../support/runtime/identity/owner_plugin.rs"]
mod owner_plugin;

pub(super) use display_name::feature_display_name;
pub(super) use id::feature_id;
pub(super) use owner_plugin::feature_owner_plugin_id;
