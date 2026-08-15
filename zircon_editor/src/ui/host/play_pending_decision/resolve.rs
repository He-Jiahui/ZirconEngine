use crate::core::editor_operation::EditorOperationSource;
use crate::core::notifications::{DecisionNotificationCenter, DecisionTicket};
use crate::core::play::PendingEditApplyBudget;
use crate::ui::host::EditorHostEventController;

use super::adapter::{ExpiredReceiptRecovery, PlayPendingReceiptConsumeError};
use super::{
    PlayPendingDecisionSelection, PlayPendingEditApplyFailure, PlayPendingEditDecisionOutcome,
    PLAY_PENDING_EDITS_APPLY_OPTION, PLAY_PENDING_EDITS_DISCARD_OPTION,
};

impl EditorHostEventController {
    /// Consumes durable core Decision receipts from the retained, headless, and replay paths.
    pub(crate) fn pump_pending_play_decision_receipts(&self) -> Result<usize, String> {
        let center = self
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?;
        Ok(self.consume_pending_play_decision_receipts(center)?.len())
    }

    pub(crate) fn resolve_pending_play_decision(
        &self,
        selection_id: &str,
    ) -> Result<PlayPendingEditDecisionOutcome, String> {
        let center = self
            .context()
            .notifications()
            .decisions()
            .map_err(|error| error.to_string())?;
        let receipt = self
            .play_pending_decisions()
            .resolve(center, selection_id)?;
        let resolved_ticket = receipt.receipt().ticket().clone();
        let consumed = self
            .consume_pending_play_decision_receipts(center)?
            .into_iter()
            .find_map(|(ticket, outcome)| (ticket == resolved_ticket).then_some(outcome));
        if !receipt.newly_resolved() {
            return Ok(consumed.unwrap_or_else(|| {
                PlayPendingEditDecisionOutcome::AlreadyResolved {
                    selected_option: receipt.receipt().option_id().as_str().to_string(),
                }
            }));
        }
        consumed.ok_or_else(|| {
            format!(
                "resolved pending play-decision receipt `{}` was not consumed",
                resolved_ticket.notification_id()
            )
        })
    }

    fn consume_pending_play_decision_receipts(
        &self,
        center: &DecisionNotificationCenter,
    ) -> Result<Vec<(DecisionTicket, PlayPendingEditDecisionOutcome)>, String> {
        let mut recovery_attempted = false;
        loop {
            match self
                .play_pending_decisions()
                .consume_resolved_receipts(center, |selection| {
                    self.execute_pending_play_decision_selection(center, selection)
                }) {
                Ok(outcomes) => {
                    self.reconcile_pending_play_decision(center).map_err(|error| {
                        format!(
                            "pending play-edit receipt effect committed but prompt reconciliation failed: {error}"
                        )
                    })?;
                    return Ok(outcomes);
                }
                Err(PlayPendingReceiptConsumeError::Dispatch(error)) => return Err(error),
                Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => {
                    let recovery = self
                        .play_pending_decisions()
                        .recover_expired_receipts(center, resume_cursor, |stale_cutoff| {
                            self.republish_pending_play_decision_after_expiry(center, stale_cutoff)
                        })
                        .map_err(|error| {
                            format!(
                                "pending play-edit receipt replay expired and recovery failed: {error}"
                            )
                        })?;
                    match recovery {
                        ExpiredReceiptRecovery::ReplacementPublished => {
                            return Err(
                                "pending play-edit receipt replay lost an unconsumed Apply/Discard choice; a new Decision was published and must be selected explicitly"
                                    .to_string(),
                            );
                        }
                        ExpiredReceiptRecovery::CursorAdvanced { .. } if !recovery_attempted => {
                            recovery_attempted = true;
                        }
                        ExpiredReceiptRecovery::CursorAdvanced { .. } => return Ok(Vec::new()),
                    }
                }
            }
        }
    }

    fn execute_pending_play_decision_selection(
        &self,
        center: &DecisionNotificationCenter,
        selection: &PlayPendingDecisionSelection,
    ) -> Result<PlayPendingEditDecisionOutcome, String> {
        match selection.option_id().as_str() {
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
                    .map(|failure| PlayPendingEditApplyFailure::new(failure.intent, failure.error))
                    .collect::<Vec<_>>();
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
