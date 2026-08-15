use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[cfg(test)]
use std::sync::Arc;
use std::sync::Mutex;

use crate::core::i18n::EditorI18nService;
use crate::core::notifications::{
    present_decision, DecisionNotification, DecisionNotificationCenter, DecisionNotificationError,
    DecisionOption, DecisionOptionId, DecisionReceiptCursor, DecisionReceiptSequence,
    DecisionResolveReport, DecisionTicket, NotificationId, NotificationSource,
};
use crate::core::play::PendingEditDecisionPrompt;

use super::{
    PlayPendingDecisionOption, PlayPendingDecisionSelection, PlayPendingEditDecisionOutcome,
    PLAY_PENDING_EDITS_APPLY_OPTION, PLAY_PENDING_EDITS_DISCARD_OPTION,
};

// Keep selection mappings for the same bounded horizon as core receipt replay.
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
    selections: BTreeMap<DecisionOptionId, String>,
}

#[derive(Debug)]
pub(super) enum PlayPendingReceiptConsumeError {
    CursorExpired {
        resume_cursor: DecisionReceiptCursor,
    },
    Dispatch(String),
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
    ) -> Result<bool, String> {
        self.publish_with_stale_cutoff(center, prompt, None)
    }

    pub(super) fn publish_replacement_after_expiry(
        &self,
        center: &DecisionNotificationCenter,
        prompt: &PendingEditDecisionPrompt,
        stale_cutoff: DecisionReceiptCursor,
    ) -> Result<bool, String> {
        self.publish_with_stale_cutoff(center, prompt, Some(stale_cutoff))
    }

    fn publish_with_stale_cutoff(
        &self,
        center: &DecisionNotificationCenter,
        prompt: &PendingEditDecisionPrompt,
        stale_cutoff: Option<DecisionReceiptCursor>,
    ) -> Result<bool, String> {
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
        state.next_sequence = state.next_sequence.checked_add(1).ok_or_else(|| {
            "play pending-edit decision identifier space is exhausted".to_string()
        })?;
        let notification_id = NotificationId::parse(format!(
            "{PLAY_PENDING_EDITS_NOTIFICATION_PREFIX}.{sequence}"
        ))
        .map_err(|error| error.to_string())?;
        let apply_option = DecisionOptionId::parse(PLAY_PENDING_EDITS_APPLY_OPTION)
            .map_err(|error| error.to_string())?;
        let discard_option = DecisionOptionId::parse(PLAY_PENDING_EDITS_DISCARD_OPTION)
            .map_err(|error| error.to_string())?;
        let notification = DecisionNotification::new(
            notification_id.clone(),
            NotificationSource::builtin("editor.play").map_err(|error| error.to_string())?,
            "editor.play.pending_edits.title",
            "editor.play.pending_edits.message",
            vec![
                DecisionOption::new(apply_option.clone(), "editor.play.pending_edits.apply")
                    .map_err(|error| error.to_string())?,
                DecisionOption::new(discard_option.clone(), "editor.play.pending_edits.discard")
                    .map_err(|error| error.to_string())?,
            ],
        )
        .map_err(|error| error.to_string())?
        .with_message_argument("pending_count", prompt.pending_count as u64)
        .map_err(|error| error.to_string())?
        .with_message_argument("payload_bytes", prompt.payload_bytes as u64)
        .map_err(|error| error.to_string())?
        .with_message_argument(
            "oldest_age_secs",
            prompt.oldest_age.map_or(0, |age| age.as_secs()),
        )
        .map_err(|error| error.to_string())?;
        let ticket = center
            .publish(notification)
            .map_err(|error| error.to_string())?;
        state
            .completed_receipt_tickets
            .extend(stale_receipt_tickets);
        state.decisions.insert(
            notification_id.clone(),
            TrackedPlayDecision {
                ticket,
                selections: BTreeMap::from([
                    (
                        apply_option,
                        format!(
                            "play_pending_decision_{sequence}_{PLAY_PENDING_EDITS_APPLY_OPTION}"
                        ),
                    ),
                    (
                        discard_option,
                        format!(
                            "play_pending_decision_{sequence}_{PLAY_PENDING_EDITS_DISCARD_OPTION}"
                        ),
                    ),
                ]),
            },
        );
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

    pub(super) fn pending_options(
        &self,
        center: &DecisionNotificationCenter,
        i18n: &EditorI18nService,
    ) -> Vec<PlayPendingDecisionOption> {
        let pending = center.pending_snapshot();
        let decisions = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .decisions
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let mut options = Vec::new();
        for decision in decisions {
            let Some(snapshot) = pending
                .iter()
                .find(|snapshot| snapshot.ticket() == &decision.ticket)
            else {
                continue;
            };
            let presentation = present_decision(snapshot, i18n);
            for option in presentation.options() {
                let Some(selection_id) = decision.selections.get(option.id()) else {
                    continue;
                };
                options.push(PlayPendingDecisionOption::new(
                    selection_id.clone(),
                    presentation.ticket().clone(),
                    option.id().clone(),
                    presentation.title().to_string(),
                    format!("{} [{}]", presentation.message(), option.label()),
                ));
            }
        }
        options
    }

    pub(super) fn selection(&self, selection_id: &str) -> Option<PlayPendingDecisionSelection> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .decisions
            .values()
            .find_map(|decision| {
                decision
                    .selections
                    .iter()
                    .find_map(|(option_id, recorded_selection_id)| {
                        (recorded_selection_id == selection_id).then(|| {
                            PlayPendingDecisionSelection::new(
                                decision.ticket.clone(),
                                option_id.clone(),
                            )
                        })
                    })
            })
    }

    pub(super) fn resolve(
        &self,
        center: &DecisionNotificationCenter,
        selection_id: &str,
    ) -> Result<DecisionResolveReport, String> {
        let selection = self
            .selection(selection_id)
            .ok_or_else(|| format!("unknown pending play-decision selection `{selection_id}`"))?;
        center
            .resolve(selection.ticket(), selection.option_id())
            .map_err(|error| error.to_string())
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
        ) -> Result<PlayPendingEditDecisionOutcome, String>,
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
            error => PlayPendingReceiptConsumeError::Dispatch(error.to_string()),
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
        republish: impl FnOnce(DecisionReceiptCursor) -> Result<ExpiredReceiptRepublish, String>,
    ) -> Result<ExpiredReceiptRecovery, String> {
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
                Err(
                    "an owned Apply/Discard choice was lost and no pending prompt is available for explicit replacement"
                        .to_string(),
                )
            }
            ExpiredReceiptRepublish::ExistingDecision => {
                let stale_cutoff = self.refresh_expired_recovery_cutoff(center, stale_cutoff)?;
                let classification = self.expired_receipt_classification(center, stale_cutoff);
                if classification.replacement_required {
                    return Err(
                        "expired pending play-edit receipt recovery did not establish a replacement Decision"
                            .to_string(),
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
    ) -> Result<DecisionReceiptCursor, String> {
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
            Err(error) => {
                return Err(format!(
                    "failed to establish expired-receipt recovery cursor: {error}"
                ));
            }
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
            .find(|decision| {
                &decision.ticket == ticket && decision.selections.contains_key(option_id)
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
