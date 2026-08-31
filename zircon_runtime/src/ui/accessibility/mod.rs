pub(crate) use action::dispatch_accessibility_action;
pub(crate) use budget::{AccessibilityBuildBudget, AccessibilitySnapshotBudgetError};
pub(crate) use extract::{accessibility_snapshot, accessibility_snapshot_bounded};

mod action;
mod budget;
mod diagnostics;
mod extract;
mod name;
mod semantic_text;

#[cfg(feature = "accessibility-accesskit")]
pub(crate) mod accesskit;
