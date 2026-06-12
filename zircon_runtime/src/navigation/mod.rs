//! Built-in navigation fallback used by dynamic runtime sessions.

mod module;
mod runtime;

pub use module::{module_descriptor, BuiltinNavigationModule, BUILTIN_NAVIGATION_MODULE_NAME};
pub use runtime::BuiltinNavigationManager;
