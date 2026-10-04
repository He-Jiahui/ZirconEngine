//! Runtime foundation services and shared process configuration.

mod module;
mod runtime;

pub use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;
pub use module::{bind_config_file_path, module_descriptor, FoundationModule};
pub use runtime::DefaultConfigManager;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
