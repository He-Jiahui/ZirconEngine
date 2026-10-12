#[path = "../../feedback/ui_diagnostics/observability.rs"]
mod observability;

use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    observability::feedback(action_id)
}
