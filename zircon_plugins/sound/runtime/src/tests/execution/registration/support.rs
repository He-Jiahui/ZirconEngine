#[path = "support/ids.rs"]
mod ids;
#[path = "../support/registration.rs"]
mod registration;

pub(super) use ids::{HANDLER_ID, PLUGIN_ID};
pub(super) use registration::register_dynamic_event_handler;
