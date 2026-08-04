use crate::core::notifications::DecisionNotificationCenter;
use crate::core::play::PendingEditDecisionPrompt;
use crate::ui::host::EditorHostEventController;

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
        Ok(self.play_pending_decisions().pending_options(center))
    }

    fn reconcile_pending_play_decision(
        &self,
        center: &DecisionNotificationCenter,
    ) -> Result<bool, String> {
        self.play_sessions()
            .with_pending_edit_decision_prompt(|prompt| {
                self.play_pending_decisions().publish(center, prompt)
            })
    }
}
