pub(crate) use action::dispatch_accessibility_action;
pub(crate) use extract::accessibility_snapshot;

mod action;
mod diagnostics;
mod extract;
mod name;

#[cfg(feature = "accessibility-accesskit")]
pub(crate) mod accesskit;
