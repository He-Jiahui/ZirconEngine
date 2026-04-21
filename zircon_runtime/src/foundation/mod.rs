//! Runtime foundation services and shared process configuration.

mod module;
mod runtime;

pub use module::{
    module_descriptor, FoundationModule, CONFIG_DRIVER_NAME, EVENT_DRIVER_NAME,
    FOUNDATION_MODULE_NAME,
};
pub use runtime::{ConfigDriver, DefaultConfigManager, DefaultEventManager, EventDriver};

#[cfg(test)]
mod tests;
