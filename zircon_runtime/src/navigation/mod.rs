//! Built-in navigation fallback used by dynamic runtime sessions.

#[path = "module.rs"]
mod module;
#[path = "operation/mod.rs"]
mod operation;
#[path = "repath_budget.rs"]
mod repath_budget;
#[path = "runtime.rs"]
mod runtime;

pub use module::{module_descriptor, BuiltinNavigationModule, BUILTIN_NAVIGATION_MODULE_NAME};
pub use operation::register_navigation_operation_handlers;
pub use repath_budget::NavRepathBudget;
pub use runtime::BuiltinNavigationManager;
