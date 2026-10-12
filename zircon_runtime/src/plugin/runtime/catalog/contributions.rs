#[path = "contributions/diagnostic.rs"]
mod diagnostic;
#[path = "contributions/extension.rs"]
mod extension;

pub(super) use diagnostic::push_runtime_extension_result;
pub(super) use extension::merge_extension_registry_contributions;
pub(super) use extension::merge_extension_registry_contributions_for_runtime_modules;
