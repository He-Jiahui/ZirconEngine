use crate::core::notifications::{DecisionNotificationCenter, DecisionReceiptCursor};
use crate::ui::host::EditorHostEventController;

use super::adapter::ExpiredReceiptRepublish;
use super::PlayPendingDecisionPublishError;

impl EditorHostEventController {
    pub(crate) fn reconcile_pending_play_decision_from_controller(
        &self,
    ) -> Result<bool, PlayPendingDecisionPublishError> {
        let center = self.context().notifications().decisions()?;
        self.reconcile_pending_play_decision(center)
    }

    pub(super) fn reconcile_pending_play_decision(
        &self,
        center: &DecisionNotificationCenter,
    ) -> Result<bool, PlayPendingDecisionPublishError> {
        let mut published = false;
        let prompt_available =
            self.play_sessions()
                .with_pending_edit_decision_prompt(|prompt| {
                    published = self.play_pending_decisions().publish(center, prompt)?;
                    Ok::<(), PlayPendingDecisionPublishError>(())
                })?;
        Ok(prompt_available && published)
    }

    pub(super) fn republish_pending_play_decision_after_expiry(
        &self,
        center: &DecisionNotificationCenter,
        stale_cutoff: DecisionReceiptCursor,
    ) -> Result<ExpiredReceiptRepublish, PlayPendingDecisionPublishError> {
        let mut outcome = ExpiredReceiptRepublish::NotRequired;
        let prompt_available =
            self.play_sessions()
                .with_pending_edit_decision_prompt(|prompt| {
                    outcome = if self
                        .play_pending_decisions()
                        .publish_replacement_after_expiry(center, prompt, stale_cutoff)?
                    {
                        ExpiredReceiptRepublish::Published
                    } else {
                        ExpiredReceiptRepublish::ExistingDecision
                    };
                    Ok::<(), PlayPendingDecisionPublishError>(())
                })?;
        Ok(if prompt_available {
            outcome
        } else {
            ExpiredReceiptRepublish::NotRequired
        })
    }
}
