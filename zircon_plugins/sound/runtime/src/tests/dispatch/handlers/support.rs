#[path = "support/ids.rs"]
mod ids;
#[path = "support/registration.rs"]
mod registration;
#[path = "support/submission.rs"]
mod submission;

pub(super) use registration::{register_ambient_event, register_ambient_handler};
pub(super) use submission::submit_ambient_event;
