#[path = "project_policy/dependencies.rs"]
mod dependencies;
#[path = "project_policy/selection.rs"]
mod selection;
#[path = "project_policy/status.rs"]
mod status;
#[path = "project_policy/transitions.rs"]
mod transitions;

pub(super) use dependencies::feature_dependency_enable_message;
pub(super) use selection::current_native_aware_project_selection;
pub(super) use status::{packaging_status_label, target_modes_status_label};
pub(super) use transitions::{next_packaging, next_target_modes};

#[cfg(test)]
#[path = "project_policy/tests/cases.rs"]
mod tests;
