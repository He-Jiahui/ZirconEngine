use std::time::Duration;

/// Bounded-ingress and retained-poll state for the UI asset workspace watcher.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiAssetWorkspaceWatchDiagnostics {
    pub pending_path_count: usize,
    pub reconcile_cursor_active: bool,
    pub received_path_count: u64,
    pub coalesced_path_count: u64,
    pub overflow_count: u64,
    pub oldest_pending_age: Duration,
    pub budget_exhausted: bool,
    pub refresh_pending_asset_count: usize,
    pub refresh_active: bool,
    pub refresh_deferred_retry_count: usize,
    pub refresh_exhausted_retry_count: u64,
    pub refresh_superseded_count: u64,
}

/// One retained-poll result plus the observable state left for later ticks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetWorkspaceWatchPollReport {
    pub changed_asset_ids: Vec<String>,
    pub diagnostics: UiAssetWorkspaceWatchDiagnostics,
}
