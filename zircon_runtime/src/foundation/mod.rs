//! Runtime foundation services and shared process configuration.

#[path = "module.rs"]
mod module;
#[cfg(feature = "native-host")]
#[path = "runtime/mod.rs"]
mod runtime;
#[cfg(not(feature = "native-host"))]
#[path = "runtime/portable.rs"]
mod runtime;

pub use crate::core::framework::foundation::FOUNDATION_MODULE_NAME;
#[cfg(feature = "native-host")]
pub use module::bind_config_file_path;
pub use module::{module_descriptor, FoundationModule};
pub use runtime::DefaultConfigManager;

#[cfg(all(test, feature = "native-host"))]
#[path = "tests/cases.rs"]
mod tests;
