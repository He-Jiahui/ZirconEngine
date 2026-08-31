//! Runtime foundation services and shared process configuration.

mod module;
mod runtime;

pub use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;
pub use module::{module_descriptor, FoundationModule};
pub use runtime::{DefaultConfigManager, DefaultEventManager};

#[cfg(test)]
mod tests;
