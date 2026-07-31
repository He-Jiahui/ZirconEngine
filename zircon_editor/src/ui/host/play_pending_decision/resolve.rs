use crate::core::editor_operation::EditorOperationSource;
use crate::core::play::PendingEditApplyBudget;
use crate::ui::host::EditorHostEventController;

use super::{
    PlayPendingEditApplyFailure, PlayPendingEditDecisionOutcome, PLAY_PENDING_EDITS_APPLY_OPTION,
    PLAY_PENDING_EDITS_DISCARD_OPTION,
};

impl EditorHostEventController {
    pub(crate) fn resolve_pending_play_decision(
        &self,
        selection_id: &str,
    ) -> Result<PlayPendingEditDecisionOutcome, String> {
        let center = self
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?;
        let receipt = self.play_pending_decisions.resolve(center, selection_id)?;
        if !receipt.newly_resolved() {
            return Ok(PlayPendingEditDecisionOutcome::AlreadyResolved {
                selected_option: receipt.receipt().option_id().as_str().to_string(),
            });
        }

        match receipt.receipt().option_id().as_str() {
            PLAY_PENDING_EDITS_APPLY_OPTION => {
                let report = self
                    .play_sessions()
                    .apply_pending_edits(PendingEditApplyBudget::interactive(), |intent| {
                        let operation_id = intent.invocation.operation_id.to_string();
                        let record = self.invoke_operation(
                            EditorOperationSource::UiBinding,
                            intent.invocation.as_ref().clone(),
                        )?;
                        match record.result.error.as_deref().map(str::trim) {
                            Some(error) if !error.is_empty() => Err(format!(
                                "operation {operation_id} reported a control failure: {error}"
                            )),
                            _ => Ok(()),
                        }
                    })
                    .map_err(|error| format!("failed to apply queued play edits: {error}"))?;
                let failures = report
                    .failures
                    .into_iter()
                    .map(|failure| {
                        PlayPendingEditApplyFailure::new(
                            failure.intent.id.value(),
                            failure.intent.invocation.operation_id.to_string(),
                            failure.error,
                        )
                    })
                    .collect::<Vec<_>>();
                self.publish_pending_edit_decision(
                    self.play_sessions().pending_edit_decision_prompt().as_ref(),
                )?;
                Ok(PlayPendingEditDecisionOutcome::Applied {
                    applied_count: report.applied.len(),
                    failures,
                })
            }
            PLAY_PENDING_EDITS_DISCARD_OPTION => {
                let report = self
                    .play_sessions()
                    .discard_pending_edits()
                    .map_err(|error| format!("failed to discard queued play edits: {error}"))?;
                Ok(PlayPendingEditDecisionOutcome::Discarded {
                    discarded_count: report.discarded_count,
                })
            }
            option => Err(format!(
                "unsupported pending play-decision option `{option}`"
            )),
        }
    }
}
