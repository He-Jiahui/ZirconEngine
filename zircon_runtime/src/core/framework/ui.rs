//! Shared UI-facing framework contracts land here as editor/runtime UI boundaries converge.

#[path = "ui/module_identity.rs"]
mod module_identity;

pub use module_identity::UI_MODULE_NAME;
