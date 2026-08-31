use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[cfg(test)]
use std::sync::Arc;
use std::sync::Mutex;

use crate::core::notifications::{
    DecisionNotification, DecisionNotificationCenter, DecisionNotificationError, DecisionOption,
    DecisionOptionId, DecisionReceiptCursor, DecisionReceiptSequence, DecisionTicket,
    NotificationId, NotificationSource,
};
use crate::core::play::PendingEditDecisionPrompt;

use super::{
    PlayPendingDecisionPublishError, PlayPendingDecisionReceiptDispatchError,
    PlayPendingDecisionReceiptRecoveryError, PlayPendingDecisionSelection,
    PlayPendingEditDecisionOutcome, PLAY_PENDING_EDITS_APPLY_OPTION,
    PLAY_PENDING_EDITS_DISCARD_OPTION,
};

// Retain owned ticket identities for the same bounded horizon as core receipt replay.
const MAX_RETAINED_DECISIONS: usize = 256;
const PLAY_PENDING_EDITS_NOTIFICATION_PREFIX: &str = "editor.play.pending_edits";

#[derive(Default)]
pub(crate) struct PlayPendingEditDecisionAdapter {
    state: Mutex<PlayPendingEditDecisionState>,
    receipt_gate: Mutex<()>,
    #[cfg(test)]
    before_publish_state_lock_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

#[derive(Default)]
struct PlayPendingEditDecisionState {
    next_sequence: u64,
    decisions: BTreeMap<NotificationId, TrackedPlayDecision>,
    decision_order: VecDeque<NotificationId>,
    receipt_cursor: Option<DecisionReceiptCursor>,
    expired_recovery_cutoff: Option<DecisionReceiptCursor>,
    completed_receipts: BTreeMap<DecisionReceiptSequence, PlayPendingEditDecisionOutcome>,
    completed_receipt_tickets: BTreeSet<DecisionTicket>,
}

#[derive(Clone)]
struct TrackedPlayDecision {
    ticket: DecisionTicket,
}

#[derive(Debug)]
pub(super) enum PlayPendingReceiptConsumeError {
    CursorExpired {
        resume_cursor: DecisionReceiptCursor,
    },
    Decision(DecisionNotificationError),
    Dispatch(PlayPendingDecisionReceiptDispatchError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExpiredReceiptRepublish {
    Published,
    NotRequired,
    ExistingDecision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExpiredReceiptRecovery {
    CursorAdvanced { owned_receipt_after_cutoff: bool },
    ReplacementPublished,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExpiredReceiptClassification {
    replacement_required: bool,
    owned_receipt_after_cutoff: bool,
}

impl PlayPendingEditDecisionAdapter {
    pub(super) fn publish(
        &self,
        center: &DecisionNotificationCenter,
        prompt: &PendingEditDecisionPrompt,
    ) -> Result<bool, PlayPendingDecisionPublishError> {
        self.publish_with_stale_cutoff(center, prompt, None)
    }

    pub(super) fn publish_replacement_after_expiry(
        &self,
        center: &DecisionNotificationCenter,
        prompt: &PendingEditDecisionPrompt,
        stale_cutoff: DecisionReceiptCursor,
    ) -> Result<bool, PlayPendingDecisionPublishError> {
        self.publish_with_stale_cutoff(center, prompt, Some(stale_cutoff))
    }

    fn publish_with_stale_cutoff(
        &self,
        center: &DecisionNotificationCenter,
        prompt: &PendingEditDecisionPrompt,
        stale_cutoff: Option<DecisionReceiptCursor>,
    ) -> Result<bool, PlayPendingDecisionPublishError> {
        #[cfg(test)]
        self.run_before_publish_state_lock_hook();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let decisions = center.snapshot();
        let tracked_tickets = state
            .decisions
            .values()
            .map(|decision| decision.ticket.clone())
            .collect::<BTreeSet<_>>();
        let mut stale_receipt_tickets = stale_cutoff
            .map(|cutoff| {
                decisions
                    .iter()
                    .filter_map(|snapshot| {
                        snapshot
                            .resolved()
                            .filter(|receipt| {
                                tracked_tickets.contains(receipt.ticket())
                                    && receipt.sequence().value() <= cutoff.value()
                            })
                            .map(|receipt| receipt.ticket().clone())
                    })
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        if stale_cutoff.is_some() {
            stale_receipt_tickets.extend(
                state
                    .decisions
                    .values()
                    .filter(|tracked| {
                        !decisions
                            .iter()
                            .any(|snapshot| snapshot.ticket() == &tracked.ticket)
                    })
                    .map(|tracked| tracked.ticket.clone()),
            );
        }
        if state.decisions.values().any(|decision| {
            decisions.iter().any(|snapshot| {
                snapshot.ticket() == &decision.ticket
                    && (snapshot.resolved().is_none()
                        || !(state.completed_receipt_tickets.contains(&decision.ticket)
                            || stale_receipt_tickets.contains(&decision.ticket)))
            })
        }) {
            return Ok(false);
        }

        let sequence = state.next_sequence;
        state.next_sequence = state
            .next_sequence
            .checked_add(1)
            .ok_or(PlayPendingDecisionPublishError::SequenceExhausted)?;
        let notification_id = NotificationId::parse(format!(
            "{PLAY_PENDING_EDITS_NOTIFICATION_PREFIX}.{sequence}"
        ))?;
        let apply_option = DecisionOptionId::parse(PLAY_PENDING_EDITS_APPLY_OPTION)?;
        let discard_option = DecisionOptionId::parse(PLAY_PENDING_EDITS_DISCARD_OPTION)?;
        let notification = DecisionNotification::new(
            notification_id.clone(),
            NotificationSource::builtin("editor.play")?,
            "editor.play.pending_edits.title",
            "editor.play.pending_edits.message",
            vec![
                DecisionOption::new(apply_option.clone(), "editor.play.pending_edits.apply")?,
                DecisionOption::new(discard_option.clone(), "editor.play.pending_edits.discard")?,
            ],
        )?
        .with_message_argument("pending_count", prompt.pending_count as u64)?
        .with_message_argument("payload_bytes", prompt.payload_bytes as u64)?
        .with_message_argument(
            "oldest_age_secs",
            prompt.oldest_age.map_or(0, |age| age.as_secs()),
        )?;
        let ticket = center.publish(notification)?;
        state
            .completed_receipt_tickets
            .extend(stale_receipt_tickets);
        state
            .decisions
            .insert(notification_id.clone(), TrackedPlayDecision { ticket });
        state.decision_order.push_back(notification_id);
        while state.decision_order.len() > MAX_RETAINED_DECISIONS {
            let Some(expired) = state.decision_order.pop_front() else {
                break;
            };
            if let Some(decision) = state.decisions.remove(&expired) {
                state.completed_receipt_tickets.remove(&decision.ticket);
            }
        }
        Ok(true)
    }

    /// Drains core Decision receipts exactly once for the Play tickets owned by this adapter.
    ///
    /// The handler runs outside the adapter state lock. If a later receipt fails, successful
    /// earlier effects are retained until the batch commits its cursor, preventing retries from
    /// applying or discarding the same pending edits twice.
    pub(super) fn consume_resolved_receipts(
        &self,
        center: &DecisionNotificationCenter,
        mut consume: impl FnMut(
            &PlayPendingDecisionSelection,
        ) -> Result<
            PlayPendingEditDecisionOutcome,
            PlayPendingDecisionReceiptDispatchError,
        >,
    ) -> Result<Vec<(DecisionTicket, PlayPendingEditDecisionOutcome)>, PlayPendingReceiptConsumeError>
    {
        let _receipt_gate = self
            .receipt_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let cursor = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .receipt_cursor
            .unwrap_or_else(|| center.initial_cursor());
        let batch = center.receipts_since(cursor).map_err(|error| match error {
            DecisionNotificationError::CursorExpired { resume_cursor, .. } => {
                PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }
            }
            error => PlayPendingReceiptConsumeError::Decision(error),
        })?;

        let mut outcomes = Vec::new();
        for receipt in batch.receipts() {
            let ticket = receipt.ticket().clone();
            let Some(selection) = self.selection_for_receipt(&ticket, receipt.option_id()) else {
                continue;
            };
            let completed = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .completed_receipts
                .get(&receipt.sequence())
                .cloned();
            let outcome = match completed {
                Some(outcome) => outcome,
                None => {
                    let outcome =
                        consume(&selection).map_err(PlayPendingReceiptConsumeError::Dispatch)?;
                    self.state
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .completed_receipts
                        .insert(receipt.sequence(), outcome.clone());
                    outcome
                }
            };
            outcomes.push((ticket, outcome));
        }

        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // A core receipt keeps its Decision authoritative until this adapter commits the
        // receipt cursor. Only then may a still-pending prompt publish a replacement choice.
        state
            .completed_receipt_tickets
            .extend(outcomes.iter().map(|(ticket, _)| ticket.clone()));
        state.receipt_cursor = Some(batch.next_cursor());
        state.completed_receipts.clear();
        Ok(outcomes)
    }

    /// Advances an expired cursor without guessing an evicted Play choice.
    ///
    /// Foreign-only expiry and an existing live/post-cutoff Play choice can advance directly.
    /// An evicted owned choice always requires a replacement Decision; prompt disappearance is an
    /// integrity error rather than permission to guess or discard the lost selection.
    pub(super) fn recover_expired_receipts(
        &self,
        center: &DecisionNotificationCenter,
        resume_cursor: DecisionReceiptCursor,
        republish: impl FnOnce(
            DecisionReceiptCursor,
        )
            -> Result<ExpiredReceiptRepublish, PlayPendingDecisionPublishError>,
    ) -> Result<ExpiredReceiptRecovery, PlayPendingDecisionReceiptRecoveryError> {
        let _receipt_gate = self
            .receipt_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let stale_cutoff = self.refresh_expired_recovery_cutoff(center, resume_cursor)?;
        let classification = self.expired_receipt_classification(center, stale_cutoff);
        if !classification.replacement_required {
            self.commit_expired_receipt_cutoff(stale_cutoff);
            return Ok(ExpiredReceiptRecovery::CursorAdvanced {
                owned_receipt_after_cutoff: classification.owned_receipt_after_cutoff,
            });
        }

        match republish(stale_cutoff)? {
            ExpiredReceiptRepublish::Published => {
                let stale_cutoff = self.refresh_expired_recovery_cutoff(center, stale_cutoff)?;
                self.commit_expired_receipt_cutoff(stale_cutoff);
                Ok(ExpiredReceiptRecovery::ReplacementPublished)
            }
            ExpiredReceiptRepublish::NotRequired => {
                Err(PlayPendingDecisionReceiptRecoveryError::ReplacementPromptUnavailable)
            }
            ExpiredReceiptRepublish::ExistingDecision => {
                let stale_cutoff = self.refresh_expired_recovery_cutoff(center, stale_cutoff)?;
                let classification = self.expired_receipt_classification(center, stale_cutoff);
                if classification.replacement_required {
                    return Err(
                        PlayPendingDecisionReceiptRecoveryError::ReplacementDecisionNotEstablished,
                    );
                }
                self.commit_expired_receipt_cutoff(stale_cutoff);
                Ok(ExpiredReceiptRecovery::CursorAdvanced {
                    owned_receipt_after_cutoff: classification.owned_receipt_after_cutoff,
                })
            }
        }
    }

    fn refresh_expired_recovery_cutoff(
        &self,
        center: &DecisionNotificationCenter,
        requested_resume: DecisionReceiptCursor,
    ) -> Result<DecisionReceiptCursor, DecisionNotificationError> {
        let candidate = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .expired_recovery_cutoff
            .filter(|cutoff| cutoff.value() >= requested_resume.value())
            .unwrap_or(requested_resume);
        let cutoff = match center.receipts_since(candidate) {
            Ok(_) => candidate,
            Err(DecisionNotificationError::CursorExpired { resume_cursor, .. }) => resume_cursor,
            Err(error) => return Err(error),
        };
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .expired_recovery_cutoff = Some(cutoff);
        Ok(cutoff)
    }

    fn expired_receipt_classification(
        &self,
        center: &DecisionNotificationCenter,
        stale_cutoff: DecisionReceiptCursor,
    ) -> ExpiredReceiptClassification {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let decisions = center.snapshot();
        let mut lost_owned_choice = false;
        let mut has_current_owned_choice = false;
        let mut owned_receipt_after_cutoff = false;
        for tracked in state.decisions.values() {
            if state.completed_receipt_tickets.contains(&tracked.ticket) {
                continue;
            }
            match decisions
                .iter()
                .find(|snapshot| snapshot.ticket() == &tracked.ticket)
            {
                Some(snapshot) => {
                    has_current_owned_choice = true;
                    owned_receipt_after_cutoff |= snapshot
                        .resolved()
                        .is_some_and(|receipt| receipt.sequence().value() > stale_cutoff.value());
                }
                None => lost_owned_choice = true,
            }
        }
        ExpiredReceiptClassification {
            replacement_required: lost_owned_choice && !has_current_owned_choice,
            owned_receipt_after_cutoff,
        }
    }

    fn commit_expired_receipt_cutoff(&self, stale_cutoff: DecisionReceiptCursor) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.receipt_cursor = Some(stale_cutoff);
        state.expired_recovery_cutoff = None;
        state.completed_receipts.clear();
    }

    fn selection_for_receipt(
        &self,
        ticket: &DecisionTicket,
        option_id: &DecisionOptionId,
    ) -> Option<PlayPendingDecisionSelection> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .decisions
            .values()
            .find(|decision| &decision.ticket == ticket)
            .filter(|_| {
                matches!(
                    option_id.as_str(),
                    PLAY_PENDING_EDITS_APPLY_OPTION | PLAY_PENDING_EDITS_DISCARD_OPTION
                )
            })
            .map(|decision| {
                PlayPendingDecisionSelection::new(decision.ticket.clone(), option_id.clone())
            })
    }

    #[cfg(test)]
    pub(super) fn configure_before_publish_state_lock_hook(
        &self,
        hook: Arc<dyn Fn() + Send + Sync>,
    ) {
        *self
            .before_publish_state_lock_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(hook);
    }

    #[cfg(test)]
    fn run_before_publish_state_lock_hook(&self) {
        if let Some(hook) = self
            .before_publish_state_lock_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            hook();
        }
    }
}
