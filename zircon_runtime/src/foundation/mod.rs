//! Runtime foundation services and shared process configuration.

mod module;
mod runtime;

pub use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;
pub use module::{CONFIG_DRIVER_NAME, EVENT_DRIVER_NAME, FoundationModule, module_descriptor};
pub use runtime::{ConfigDriver, DefaultConfigManager, DefaultEventManager, EventDriver};

#[cfg(test)]
mod tests;
