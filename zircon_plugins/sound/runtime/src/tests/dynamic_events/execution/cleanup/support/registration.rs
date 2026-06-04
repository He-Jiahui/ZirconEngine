mod descriptors;
mod executor;

pub(crate) use descriptors::register_cleanup_event_and_handler;
pub(crate) use executor::register_cleanup_event_handler_and_executor;
