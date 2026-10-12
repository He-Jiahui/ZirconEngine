//! Runtime operation handlers for navigation authoring work.

#[path = "handler.rs"]
mod handler;
#[path = "registration.rs"]
mod registration;

pub use registration::register_navigation_operation_handlers;
