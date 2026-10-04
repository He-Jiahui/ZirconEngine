use crate::core::i18n::EditorI18nService;
use crate::core::notifications::{present_decision, DecisionNotificationSnapshot};

use super::ActivityDecisionSelectionId;

/// Read-only Activity projection for one option in the current operator Decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActivityDecisionOption {
    selection_id: ActivityDecisionSelectionId,
    title: String,
    message: String,
}

impl ActivityDecisionOption {
    fn new(
        selection_id: ActivityDecisionSelectionId,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            selection_id,
            title: title.into(),
            message: message.into(),
        }
    }

    pub(crate) fn selection_id(&self) -> &ActivityDecisionSelectionId {
        &self.selection_id
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

/// Projects the oldest pending Decision as one complete option group.
///
/// Presenting only the first group prevents a bounded history surface from truncating a later
/// Decision's option set. The core center owns the remaining FIFO backlog.
pub(crate) fn activity_decision_options(
    snapshots: &[DecisionNotificationSnapshot],
    i18n: &EditorI18nService,
) -> Vec<ActivityDecisionOption> {
    let Some(snapshot) = snapshots.first() else {
        return Vec::new();
    };
    let decision = present_decision(snapshot, i18n);
    let mut options = Vec::with_capacity(decision.options().len());
    options.extend(decision.options().iter().map(|option| {
        let message = match decision.display_subject() {
            Some(subject) => format!("{} ({subject}) [{}]", decision.message(), option.label()),
            None => format!("{} [{}]", decision.message(), option.label()),
        };
        ActivityDecisionOption::new(
            ActivityDecisionSelectionId::new(decision.id(), option.id()),
            decision.title(),
            message,
        )
    }));
    options
}

#[cfg(test)]
#[path = "tests/view.rs"]
mod tests;
