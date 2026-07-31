use crate::core::play::PendingEditDecisionPrompt;
use crate::ui::host::EditorHostEventController;

use super::PlayPendingDecisionOption;

impl EditorHostEventController {
    pub(crate) fn publish_pending_edit_decision(
        &self,
        prompt: Option<&PendingEditDecisionPrompt>,
    ) -> Result<bool, String> {
        let Some(prompt) = prompt else {
            return Ok(false);
        };
        let center = self
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?;
        self.play_pending_decisions.publish(center, prompt)?;
        Ok(true)
    }

    pub(crate) fn pending_play_decision_options(
        &self,
    ) -> Result<Vec<PlayPendingDecisionOption>, String> {
        let center = self
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?;
        Ok(self.play_pending_decisions.pending_options(center))
    }
}
