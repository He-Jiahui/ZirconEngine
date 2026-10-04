#![cfg(feature = "profiling")]

use std::sync::Arc;
use zr_resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManagementQuery,
    ResourceManagementScanDiagnostics, ResourceManager, ResourceRecord,
};

#[test]
fn public_scan_diagnostics_are_query_local_read_only_and_retained() {
    let manager = ResourceManager::new();
    for (label, locator, kind) in [
        ("skip-first", "res://assets/a.png", ResourceKind::Texture),
        ("emit", "res://models/b.glb", ResourceKind::Model),
        ("skip-last", "res://textures/z.png", ResourceKind::Texture),
    ] {
        manager.register_record(ResourceRecord::new(
            ResourceId::from_stable_label(label),
            kind,
            ResourceLocator::parse(locator).unwrap(),
        ));
    }
    let generation = manager.management_generation();
    let mut scan = generation.scan(ResourceManagementQuery {
        kind: Some(ResourceKind::Model),
        state: None,
    });
    let initial: ResourceManagementScanDiagnostics = scan.diagnostics();
    assert_eq!(initial.rows_emitted(), 0);
    assert_eq!(initial.filtered_rows_skipped(), 0);
    assert_eq!(initial.shard_candidate_checks(), 0);
    assert!(scan.next_row().is_some());
    let partial = scan.diagnostics();
    assert_eq!(partial.rows_emitted(), 1);
    assert_eq!(partial.filtered_rows_skipped(), 1);
    assert_eq!(partial.shard_candidate_checks(), 1);
    assert_eq!(scan.diagnostics(), partial);
    assert!(scan.next_row().is_none());
    let complete = scan.diagnostics();
    assert_eq!(complete.rows_emitted(), 1);
    assert_eq!(complete.filtered_rows_skipped(), 2);
    assert_eq!(complete.shard_candidate_checks(), 1);
    assert_eq!(partial.filtered_rows_skipped(), 1);
    assert!(scan.next_row().is_none());
    assert_eq!(scan.diagnostics(), complete);
    assert!(Arc::ptr_eq(&generation, &manager.management_generation()));
    let fresh = generation.scan(ResourceManagementQuery {
        kind: None,
        state: None,
    });
    assert_eq!(fresh.diagnostics(), initial);
}
