use thiserror::Error;

use crate::core::notifications::{DecisionNotificationError, DecisionResolveReport};
use crate::ui::activity::{ActivityDecisionSelectionError, ActivityDecisionSelectionId};

use super::EditorHostEventController;

impl EditorHostEventController {
    /// Resolves only the currently presented core Decision option.
    ///
    /// The route identifier is revalidated against the live core snapshot, so stale retained UI
    /// rows cannot resolve a replacement Decision that reused the same notification identity.
    pub(crate) fn resolve_activity_decision(
        &self,
        selection_id: &str,
    ) -> Result<DecisionResolveReport, ActivityDecisionResolutionError> {
        let selection_id = ActivityDecisionSelectionId::parse(selection_id)?;
        let selection = selection_id.selection()?;
        let center = self.context().notifications().decisions()?;
        let Some(current) = center.pending_snapshot().into_iter().next() else {
            return Err(ActivityDecisionResolutionError::NoPendingDecision);
        };
        if current.notification().id() != selection.notification_id() {
            return Err(ActivityDecisionResolutionError::NotCurrentDecision);
        }
        if !current.notification().has_option(selection.option_id()) {
            return Err(ActivityDecisionResolutionError::OptionUnavailable);
        }
        Ok(center.resolve(current.ticket(), selection.option_id())?)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ActivityDecisionResolutionError {
    #[error(transparent)]
    InvalidSelection(#[from] ActivityDecisionSelectionError),
    #[error(transparent)]
    Decision(#[from] DecisionNotificationError),
    #[error("no pending editor Decision is available for resolution")]
    NoPendingDecision,
    #[error("the submitted editor Decision is no longer current")]
    NotCurrentDecision,
    #[error("the current editor Decision does not offer the submitted option")]
    OptionUnavailable,
}

#[cfg(test)]
#[path = "tests/editor_host_activity_decision.rs"]
mod tests;
