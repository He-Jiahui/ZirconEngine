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
#[path = "tests/watch_diagnostics.rs"]
mod tests;
