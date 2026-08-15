use std::time::Duration;

use crate::asset::watch::AssetWatchBatch;

use super::ProjectAssetManager;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectAssetWatchDiagnostics {
    pub batch_count: usize,
    pub committed_generation_count: usize,
    pub reconciliation_count: usize,
    pub failed_batch_count: usize,
    pub superseded_generation_count: usize,
    pub raw_event_count: usize,
    pub effective_change_count: usize,
    pub coalesced_event_count: usize,
    pub ingress_overflow_count: usize,
    pub pending_overflow_count: usize,
    pub total_approximate_bytes: usize,
    pub max_batch_approximate_bytes: usize,
    pub max_batch_age: Duration,
    pub total_scan_import_duration: Duration,
    pub max_scan_import_duration: Duration,
    pub incremental_resource_record_count: usize,
    pub max_incremental_resource_record_count: usize,
}

impl ProjectAssetManager {
    pub fn asset_watch_diagnostics(&self) -> ProjectAssetWatchDiagnostics {
        *self
            .watch_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn record_asset_watch_batch(&self, batch: &AssetWatchBatch) {
        let mut diagnostics = self
            .watch_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        diagnostics.batch_count = diagnostics.batch_count.saturating_add(1);
        let reconciliation = if batch.requires_reconciliation { 1 } else { 0 };
        diagnostics.reconciliation_count = diagnostics
            .reconciliation_count
            .saturating_add(reconciliation);
        diagnostics.raw_event_count = diagnostics
            .raw_event_count
            .saturating_add(batch.diagnostics.raw_event_count);
        diagnostics.effective_change_count = diagnostics
            .effective_change_count
            .saturating_add(batch.changes.len());
        diagnostics.coalesced_event_count = diagnostics
            .coalesced_event_count
            .saturating_add(batch.diagnostics.coalesced_event_count);
        diagnostics.ingress_overflow_count = diagnostics
            .ingress_overflow_count
            .saturating_add(batch.diagnostics.ingress_overflow_count);
        diagnostics.pending_overflow_count = diagnostics
            .pending_overflow_count
            .saturating_add(batch.diagnostics.pending_overflow_count);
        diagnostics.total_approximate_bytes = diagnostics
            .total_approximate_bytes
            .saturating_add(batch.diagnostics.approximate_bytes);
        diagnostics.max_batch_approximate_bytes = diagnostics
            .max_batch_approximate_bytes
            .max(batch.diagnostics.approximate_bytes);
        diagnostics.max_batch_age = diagnostics.max_batch_age.max(batch.diagnostics.oldest_age);
    }

    pub(super) fn record_asset_watch_scan(&self, elapsed: Duration) {
        let mut diagnostics = self
            .watch_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        diagnostics.total_scan_import_duration = diagnostics
            .total_scan_import_duration
            .saturating_add(elapsed);
        diagnostics.max_scan_import_duration = diagnostics.max_scan_import_duration.max(elapsed);
    }

    pub(super) fn record_asset_watch_commit(&self) {
        let mut diagnostics = self
            .watch_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        diagnostics.committed_generation_count =
            diagnostics.committed_generation_count.saturating_add(1);
    }

    pub(super) fn record_asset_watch_incremental_resource_sync(&self, count: usize) {
        let mut diagnostics = self
            .watch_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        diagnostics.incremental_resource_record_count = diagnostics
            .incremental_resource_record_count
            .saturating_add(count);
        diagnostics.max_incremental_resource_record_count =
            diagnostics.max_incremental_resource_record_count.max(count);
    }

    pub(super) fn record_asset_watch_failure(&self) {
        let mut diagnostics = self
            .watch_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        diagnostics.failed_batch_count = diagnostics.failed_batch_count.saturating_add(1);
    }

    pub(super) fn record_asset_watch_superseded_generation(&self) {
        let mut diagnostics = self
            .watch_diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        diagnostics.superseded_generation_count =
            diagnostics.superseded_generation_count.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::watch::{AssetChange, AssetChangeKind, AssetWatchBatchDiagnostics};
    use crate::asset::AssetUri;

    #[test]
    fn manager_accumulates_bounded_watch_batch_diagnostics() {
        let manager = ProjectAssetManager::default();
        manager.record_asset_watch_batch(&AssetWatchBatch {
            changes: vec![AssetChange::new(
                AssetChangeKind::Modified,
                AssetUri::parse("res://data/watch.json").unwrap(),
                None,
            )],
            requires_reconciliation: true,
            diagnostics: AssetWatchBatchDiagnostics {
                raw_event_count: 3,
                coalesced_event_count: 2,
                ingress_overflow_count: 1,
                pending_overflow_count: 1,
                approximate_bytes: 128,
                oldest_age: Duration::from_millis(7),
            },
        });
        manager.record_asset_watch_scan(Duration::from_millis(4));
        manager.record_asset_watch_incremental_resource_sync(2);
        manager.record_asset_watch_commit();

        let diagnostics = manager.asset_watch_diagnostics();
        assert_eq!(diagnostics.batch_count, 1);
        assert_eq!(diagnostics.committed_generation_count, 1);
        assert_eq!(diagnostics.reconciliation_count, 1);
        assert_eq!(diagnostics.raw_event_count, 3);
        assert_eq!(diagnostics.effective_change_count, 1);
        assert_eq!(diagnostics.coalesced_event_count, 2);
        assert_eq!(diagnostics.ingress_overflow_count, 1);
        assert_eq!(diagnostics.pending_overflow_count, 1);
        assert_eq!(diagnostics.max_batch_approximate_bytes, 128);
        assert_eq!(diagnostics.max_batch_age, Duration::from_millis(7));
        assert_eq!(
            diagnostics.total_scan_import_duration,
            Duration::from_millis(4)
        );
        assert_eq!(diagnostics.incremental_resource_record_count, 2);
        assert_eq!(diagnostics.max_incremental_resource_record_count, 2);
    }
}
