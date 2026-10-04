use std::time::Duration;

use super::AssetChange;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssetWatchBatchDiagnostics {
    pub raw_event_count: usize,
    pub coalesced_event_count: usize,
    pub ingress_overflow_count: usize,
    pub pending_overflow_count: usize,
    pub approximate_bytes: usize,
    pub oldest_age: Duration,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 一次监听刷新请求；requires_reconciliation 优先于局部 changes，表示增量事件不再可信。
pub struct AssetWatchBatch {
    pub changes: Vec<AssetChange>,
    pub requires_reconciliation: bool,
    pub diagnostics: AssetWatchBatchDiagnostics,
}

impl AssetWatchBatch {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty() && !self.requires_reconciliation
    }
}
