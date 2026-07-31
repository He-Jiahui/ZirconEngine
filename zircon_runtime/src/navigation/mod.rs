//! Built-in navigation fallback used by dynamic runtime sessions.

mod module;
mod operation;
mod repath_budget;
mod runtime;

pub use module::{module_descriptor, BuiltinNavigationModule, BUILTIN_NAVIGATION_MODULE_NAME};
pub use operation::register_navigation_operation_handlers;
pub use repath_budget::NavRepathBudget;
pub use runtime::BuiltinNavigationManager;
