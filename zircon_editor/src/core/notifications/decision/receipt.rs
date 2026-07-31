use super::{DecisionOptionId, DecisionReceiptCursor, DecisionReceiptSequence, DecisionTicket};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionReceipt {
    pub(super) sequence: DecisionReceiptSequence,
    pub(super) ticket: DecisionTicket,
    pub(super) option_id: DecisionOptionId,
}

impl DecisionReceipt {
    pub const fn sequence(&self) -> DecisionReceiptSequence {
        self.sequence
    }

    pub const fn ticket(&self) -> &DecisionTicket {
        &self.ticket
    }

    pub const fn option_id(&self) -> &DecisionOptionId {
        &self.option_id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionResolveReport {
    pub(super) receipt: DecisionReceipt,
    pub(super) newly_resolved: bool,
}

impl DecisionResolveReport {
    pub const fn receipt(&self) -> &DecisionReceipt {
        &self.receipt
    }

    pub const fn newly_resolved(&self) -> bool {
        self.newly_resolved
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionReceiptBatch {
    pub(super) receipts: Vec<DecisionReceipt>,
    pub(super) next_cursor: DecisionReceiptCursor,
}

impl DecisionReceiptBatch {
    pub fn receipts(&self) -> &[DecisionReceipt] {
        &self.receipts
    }

    pub const fn next_cursor(&self) -> DecisionReceiptCursor {
        self.next_cursor
    }
}
