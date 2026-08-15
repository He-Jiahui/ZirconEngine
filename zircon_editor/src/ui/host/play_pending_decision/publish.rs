use crate::core::notifications::{DecisionNotificationCenter, DecisionReceiptCursor};
use crate::core::play::PendingEditDecisionPrompt;
use crate::ui::host::EditorHostEventController;

use super::adapter::ExpiredReceiptRepublish;
use super::PlayPendingDecisionOption;

impl EditorHostEventController {
    pub(crate) fn publish_pending_edit_decision(
        &self,
        prompt: Option<&PendingEditDecisionPrompt>,
    ) -> Result<bool, String> {
        if prompt.is_none() {
            return Ok(false);
        }
        let center = self
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?;
        self.reconcile_pending_play_decision(center)
    }

    pub(crate) fn pending_play_decision_options(
        &self,
    ) -> Result<Vec<PlayPendingDecisionOption>, String> {
        let center = self
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?;
        self.reconcile_pending_play_decision(center)?;
        Ok(self
            .play_pending_decisions()
            .pending_options(center, self.context().i18n()))
    }

    pub(super) fn reconcile_pending_play_decision(
        &self,
        center: &DecisionNotificationCenter,
    ) -> Result<bool, String> {
        let mut published = false;
        let prompt_available =
            self.play_sessions()
                .with_pending_edit_decision_prompt(|prompt| {
                    published = self.play_pending_decisions().publish(center, prompt)?;
                    Ok::<(), String>(())
                })?;
        Ok(prompt_available && published)
    }

    pub(super) fn republish_pending_play_decision_after_expiry(
        &self,
        center: &DecisionNotificationCenter,
        stale_cutoff: DecisionReceiptCursor,
    ) -> Result<ExpiredReceiptRepublish, String> {
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
                    Ok::<(), String>(())
                })?;
        Ok(if prompt_available {
            outcome
        } else {
            ExpiredReceiptRepublish::NotRequired
        })
    }
}
