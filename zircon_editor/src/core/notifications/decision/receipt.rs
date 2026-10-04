//! 把已线性化的操作者选择交还生产者；回执保留精确发布票据和序号，重复解析可复用同一回执，消费端按游标读取有限历史。
use super::{DecisionOptionId, DecisionReceiptCursor, DecisionReceiptSequence, DecisionTicket};

/// 已确认选择的不可变事实；生产者根据发布票据执行后续动作，不根据显示文字重建身份。
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
