//! 协调器测试追加结果公开耐久序号；生产耐久发布仍由事务引擎的提交线性化设计负责。

/// One committed transaction that has crossed the durable journal boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentJournalAppend {
    sequence: u64,
}

impl DocumentJournalAppend {
    pub(super) const fn new(sequence: u64) -> Self {
        Self { sequence }
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}
