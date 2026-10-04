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
