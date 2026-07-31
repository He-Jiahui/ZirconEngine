use crate::core::notifications::{DecisionOptionId, DecisionTicket};

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
pub(super) struct PlayPendingDecisionSelection {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlayPendingEditApplyFailure {
    pending_edit_id: u64,
    operation_id: String,
    error: String,
}

impl PlayPendingEditApplyFailure {
    pub(super) fn new(pending_edit_id: u64, operation_id: String, error: String) -> Self {
        Self {
            pending_edit_id,
            operation_id,
            error,
        }
    }

    pub(crate) fn pending_edit_id(&self) -> u64 {
        self.pending_edit_id
    }

    pub(crate) fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub(crate) fn error(&self) -> &str {
        &self.error
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
