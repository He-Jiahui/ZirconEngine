use crate::core::notifications::{DecisionOptionId, DecisionTicket};
use crate::core::play::PendingEditIntent;

pub(crate) const PLAY_PENDING_EDITS_APPLY_OPTION: &str = "apply";
pub(crate) const PLAY_PENDING_EDITS_DISCARD_OPTION: &str = "discard";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlayPendingDecisionOption {
    selection_id: String,
    ticket: DecisionTicket,
    option_id: DecisionOptionId,
    title: String,
    message: String,
}

impl PlayPendingDecisionOption {
    pub(crate) fn new(
        selection_id: String,
        ticket: DecisionTicket,
        option_id: DecisionOptionId,
        title: String,
        message: String,
    ) -> Self {
        Self {
            selection_id,
            ticket,
            option_id,
            title,
            message,
        }
    }

    pub(crate) fn selection_id(&self) -> &str {
        &self.selection_id
    }

    pub(crate) fn option_id(&self) -> &DecisionOptionId {
        &self.option_id
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(super) fn selection(&self) -> PlayPendingDecisionSelection {
        PlayPendingDecisionSelection {
            ticket: self.ticket.clone(),
            option_id: self.option_id.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlayPendingDecisionSelection {
    ticket: DecisionTicket,
    option_id: DecisionOptionId,
}

impl PlayPendingDecisionSelection {
    pub(super) fn ticket(&self) -> &DecisionTicket {
        &self.ticket
    }

    pub(super) fn option_id(&self) -> &DecisionOptionId {
        &self.option_id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlayPendingEditApplyFailure {
    intent: PendingEditIntent,
    error: String,
}

impl PlayPendingEditApplyFailure {
    pub(super) fn new(intent: PendingEditIntent, error: String) -> Self {
        Self { intent, error }
    }

    pub(crate) fn intent(&self) -> &PendingEditIntent {
        &self.intent
    }

    pub(crate) fn error(&self) -> &str {
        &self.error
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
