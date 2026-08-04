use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;

use crate::core::notifications::{
    DecisionNotification, DecisionNotificationCenter, DecisionOption, DecisionOptionId,
    DecisionResolveReport, DecisionTicket, NotificationId, NotificationSource,
};
use crate::core::play::PendingEditDecisionPrompt;

use super::{
    PLAY_PENDING_EDITS_APPLY_OPTION, PLAY_PENDING_EDITS_DISCARD_OPTION, PlayPendingDecisionOption,
    PlayPendingDecisionSelection,
};

// Keep selection mappings for the same bounded horizon as core receipt replay.
const MAX_RETAINED_DECISIONS: usize = 256;
const PLAY_PENDING_EDITS_NOTIFICATION_PREFIX: &str = "editor.play.pending_edits";

#[derive(Default)]
pub(crate) struct PlayPendingEditDecisionAdapter {
    state: Mutex<PlayPendingEditDecisionState>,
}

#[derive(Default)]
struct PlayPendingEditDecisionState {
    next_sequence: u64,
    decisions: BTreeMap<NotificationId, TrackedPlayDecision>,
    decision_order: VecDeque<NotificationId>,
}

struct TrackedPlayDecision {
    ticket: DecisionTicket,
    options: Vec<PlayPendingDecisionOption>,
}

impl PlayPendingEditDecisionAdapter {
    pub(super) fn publish(
        &self,
        center: &DecisionNotificationCenter,
        prompt: &PendingEditDecisionPrompt,
    ) -> Result<(), String> {
        let pending = center.pending_snapshot();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.decisions.values().any(|decision| {
            pending
                .iter()
                .any(|snapshot| snapshot.ticket() == &decision.ticket)
        }) {
            return Ok(());
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
        .map_err(|error| error.to_string())?;
        let ticket = center
            .publish(notification)
            .map_err(|error| error.to_string())?;
        let pending_count = prompt.pending_count;
        let payload_bytes = prompt.payload_bytes;
        let oldest_age = prompt
            .oldest_age
            .map(|age| format!("{}s", age.as_secs()))
            .unwrap_or_else(|| "0s".to_string());
        let options = vec![
            PlayPendingDecisionOption::new(
                format!("play_pending_decision_{sequence}_{PLAY_PENDING_EDITS_APPLY_OPTION}"),
                ticket.clone(),
                apply_option,
                "Resolve queued play edits".to_string(),
                format!(
                    "Apply {pending_count} queued edit(s) before the next Play session ({payload_bytes} bytes; oldest {oldest_age})."
                ),
            ),
            PlayPendingDecisionOption::new(
                format!("play_pending_decision_{sequence}_{PLAY_PENDING_EDITS_DISCARD_OPTION}"),
                ticket.clone(),
                discard_option,
                "Resolve queued play edits".to_string(),
                format!(
                    "Discard {pending_count} queued edit(s) before the next Play session ({payload_bytes} bytes; oldest {oldest_age})."
                ),
            ),
        ];
        state.decisions.insert(
            notification_id.clone(),
            TrackedPlayDecision { ticket, options },
        );
        state.decision_order.push_back(notification_id);
        while state.decision_order.len() > MAX_RETAINED_DECISIONS {
            let Some(expired) = state.decision_order.pop_front() else {
                break;
            };
            state.decisions.remove(&expired);
        }
        Ok(())
    }

    pub(super) fn pending_options(
        &self,
        center: &DecisionNotificationCenter,
    ) -> Vec<PlayPendingDecisionOption> {
        let pending = center.pending_snapshot();
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .decisions
            .values()
            .filter(|decision| {
                pending
                    .iter()
                    .any(|snapshot| snapshot.ticket() == &decision.ticket)
            })
            .flat_map(|decision| decision.options.iter().cloned())
            .collect()
    }

    pub(super) fn selection(&self, selection_id: &str) -> Option<PlayPendingDecisionSelection> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .decisions
            .values()
            .flat_map(|decision| decision.options.iter())
            .find(|option| option.selection_id() == selection_id)
            .map(PlayPendingDecisionOption::selection)
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
}
