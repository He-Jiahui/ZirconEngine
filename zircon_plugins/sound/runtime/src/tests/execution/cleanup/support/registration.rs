#[path = "../../../support/registration/descriptors.rs"]
mod descriptors;
#[path = "../../../cleanup/support/registration/executor.rs"]
mod executor;

pub(crate) use descriptors::register_cleanup_event_and_handler;
pub(crate) use executor::register_cleanup_event_handler_and_executor;
