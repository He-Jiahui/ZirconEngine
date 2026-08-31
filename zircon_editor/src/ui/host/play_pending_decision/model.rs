use std::sync::Arc;

use crate::core::notifications::{DecisionOptionId, DecisionTicket};
use crate::core::play::PendingEditIntent;
use crate::ui::host::EditorOperationDispatchError;

pub(crate) const PLAY_PENDING_EDITS_APPLY_OPTION: &str = "apply";
pub(crate) const PLAY_PENDING_EDITS_DISCARD_OPTION: &str = "discard";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlayPendingDecisionSelection {
    ticket: DecisionTicket,
    option_id: DecisionOptionId,
}

impl PlayPendingDecisionSelection {
    pub(super) fn new(ticket: DecisionTicket, option_id: DecisionOptionId) -> Self {
        Self { ticket, option_id }
    }

    pub(super) fn ticket(&self) -> &DecisionTicket {
        &self.ticket
    }

    pub(super) fn option_id(&self) -> &DecisionOptionId {
        &self.option_id
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PlayPendingEditApplyFailure {
    intent: PendingEditIntent,
    error: Arc<EditorOperationDispatchError>,
}

impl PartialEq for PlayPendingEditApplyFailure {
    fn eq(&self, other: &Self) -> bool {
        self.intent == other.intent && self.error.to_string() == other.error.to_string()
    }
}

impl PlayPendingEditApplyFailure {
    pub(super) fn new(intent: PendingEditIntent, error: EditorOperationDispatchError) -> Self {
        Self {
            intent,
            error: Arc::new(error),
        }
    }

    pub(crate) fn intent(&self) -> &PendingEditIntent {
        &self.intent
    }

    pub(crate) fn error(&self) -> &EditorOperationDispatchError {
        self.error.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PlayPendingEditDecisionOutcome {
    Applied {
        applied_count: usize,
        failures: Vec<PlayPendingEditApplyFailure>,
    },
    Discarded {
        discarded_count: usize,
    },
    AlreadyResolved {
        selected_option: String,
    },
}
