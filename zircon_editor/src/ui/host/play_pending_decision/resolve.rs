use std::time::Duration;

use crate::core::editor_operation::EditorOperationSource;
use crate::core::notifications::{
    DecisionNotificationCenter, DecisionTicket, NotificationId, NotificationSource,
    ToastNotification, ToastSeverity,
};
use crate::core::play::PendingEditApplyBudget;
use crate::ui::host::EditorHostEventController;

use super::adapter::{ExpiredReceiptRecovery, PlayPendingReceiptConsumeError};
use super::{
    PlayPendingDecisionReceiptDispatchError, PlayPendingDecisionReceiptError,
    PlayPendingDecisionSelection, PlayPendingEditApplyFailure, PlayPendingEditDecisionOutcome,
    PLAY_PENDING_EDITS_APPLY_OPTION, PLAY_PENDING_EDITS_DISCARD_OPTION,
};

const MAX_PLAY_PENDING_FAILURE_DETAILS: usize = 4;
const PLAY_PENDING_DECISION_SUCCESS_TOAST_LIFETIME: Duration = Duration::from_millis(3_500);
const PLAY_PENDING_DECISION_FAILURE_TOAST_LIFETIME: Duration = Duration::from_secs(7);

impl EditorHostEventController {
    /// Consumes durable core Decision receipts from the retained, headless, and replay paths.
    pub(crate) fn pump_pending_play_decision_receipts(
        &self,
    ) -> Result<usize, PlayPendingDecisionReceiptError> {
        let center = self.context().notifications().decisions()?;
        Ok(self.consume_pending_play_decision_receipts(center)?.len())
    }

    fn consume_pending_play_decision_receipts(
        &self,
        center: &DecisionNotificationCenter,
    ) -> Result<
        Vec<(DecisionTicket, PlayPendingEditDecisionOutcome)>,
        PlayPendingDecisionReceiptError,
    > {
        let mut recovery_attempted = false;
        loop {
            match self
                .play_pending_decisions()
                .consume_resolved_receipts(center, |selection| {
                    self.execute_pending_play_decision_selection(center, selection)
                }) {
                Ok(outcomes) => {
                    self.publish_pending_play_decision_outcome_toasts(&outcomes);
                    self.reconcile_pending_play_decision(center)
                        .map_err(|source| PlayPendingDecisionReceiptError::Reconcile { source })?;
                    return Ok(outcomes);
                }
                Err(PlayPendingReceiptConsumeError::Decision(source)) => {
                    return Err(source.into());
                }
                Err(PlayPendingReceiptConsumeError::Dispatch(source)) => {
                    return Err(source.into());
                }
                Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => {
                    let recovery = self.play_pending_decisions().recover_expired_receipts(
                        center,
                        resume_cursor,
                        |stale_cutoff| {
                            self.republish_pending_play_decision_after_expiry(center, stale_cutoff)
                        },
                    )?;
                    match recovery {
                        ExpiredReceiptRecovery::ReplacementPublished => {
                            return Err(
                                PlayPendingDecisionReceiptError::ExplicitReplacementRequired,
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
    ) -> Result<PlayPendingEditDecisionOutcome, PlayPendingDecisionReceiptDispatchError> {
        match selection.option_id().as_str() {
            PLAY_PENDING_EDITS_APPLY_OPTION => {
                let report = self
                    .play_sessions()
                    .apply_pending_edits(PendingEditApplyBudget::interactive(), |intent| {
                        let record = self
                            .invoke_operation(
                                EditorOperationSource::UiBinding,
                                intent.invocation.as_ref().clone(),
                            )?;
                        match record.result.error.as_deref().map(str::trim) {
                            Some(message) if !message.is_empty() => Err(
                                crate::ui::host::EditorOperationDispatchError::EventDispatch(
                                    crate::ui::host::EditorEventDispatchError::Execution(
                                        crate::ui::host::EditorEventExecutionError::RecordedOperationControlFailure {
                                            operation_id: intent.invocation.operation_id.to_string(),
                                            message: message.to_string(),
                                        },
                                    ),
                                )
                                .into(),
                            ),
                            _ => Ok(()),
                        }
                    })
                    .map_err(PlayPendingDecisionReceiptDispatchError::from)?;
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
                    .map_err(PlayPendingDecisionReceiptDispatchError::from)?;
                Ok(PlayPendingEditDecisionOutcome::Discarded {
                    discarded_count: report.discarded_count,
                })
            }
            option => Err(PlayPendingDecisionReceiptDispatchError::UnsupportedOption {
                option: option.to_string(),
            }),
        }
    }

    fn publish_pending_play_decision_outcome_toasts(
        &self,
        outcomes: &[(DecisionTicket, PlayPendingEditDecisionOutcome)],
    ) {
        for (ticket, outcome) in outcomes {
            let (suffix, severity, title_key, message_key, lifetime) = match outcome {
                PlayPendingEditDecisionOutcome::Applied { failures, .. } if failures.is_empty() => {
                    (
                        "applied",
                        ToastSeverity::Success,
                        "editor.notification.pending_edits_applied.title",
                        "editor.notification.pending_edits_applied.message".to_string(),
                        PLAY_PENDING_DECISION_SUCCESS_TOAST_LIFETIME,
                    )
                }
                PlayPendingEditDecisionOutcome::Applied { failures, .. } => (
                    "apply_failed",
                    ToastSeverity::Error,
                    "editor.notification.pending_edits_failed.title",
                    pending_play_failure_toast_message(failures),
                    PLAY_PENDING_DECISION_FAILURE_TOAST_LIFETIME,
                ),
                PlayPendingEditDecisionOutcome::Discarded { .. } => (
                    "discarded",
                    ToastSeverity::Info,
                    "editor.notification.pending_edits_discarded.title",
                    "editor.notification.pending_edits_discarded.message".to_string(),
                    PLAY_PENDING_DECISION_SUCCESS_TOAST_LIFETIME,
                ),
                PlayPendingEditDecisionOutcome::AlreadyResolved { .. } => continue,
            };
            let Ok(id) = NotificationId::parse(format!("{}.{}", ticket.notification_id(), suffix))
            else {
                continue;
            };
            let Ok(source) = NotificationSource::builtin("editor.play") else {
                continue;
            };
            let Ok(notification) =
                ToastNotification::new(id, source, severity, title_key, message_key, lifetime)
            else {
                continue;
            };
            let _ = self.context().notifications().publish_toast(notification);
        }
    }
}

fn pending_play_failure_toast_message(failures: &[PlayPendingEditApplyFailure]) -> String {
    let diagnostics = failures
        .iter()
        .take(MAX_PLAY_PENDING_FAILURE_DETAILS)
        .map(|failure| {
            let error = ToastNotification::bounded_message(
                &failure.error().to_string(),
                "pending edit operation failed",
            );
            ToastNotification::bounded_message(
                &format!("pending edit intent {:?} failed: {error}", failure.intent()),
                "Queued edits could not be applied.",
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    ToastNotification::bounded_message(&diagnostics, "Queued edits could not be applied.")
}
