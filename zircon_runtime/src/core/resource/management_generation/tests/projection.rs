use super::super::*;
use crate::core::resource::ResourceLocator;
#[cfg(feature = "profiling")]
use crate::core::resource::ResourceManager;

#[test]
fn resource_management_scan_cursor_yields_matching_rows_once_in_stable_order() {
    let mut summary = ResourceManagementSummary::default();
    let mut rows_by_shard = vec![Vec::new(); RESOURCE_MANAGEMENT_SHARD_COUNT];
    for (label, locator, kind) in [
        ("scan-z", "res://scenes/z.scene.toml", ResourceKind::Scene),
        ("scan-a", "res://scenes/a.scene.toml", ResourceKind::Scene),
        ("scan-m", "res://meshes/m.mesh.toml", ResourceKind::Mesh),
    ] {
        let record = ResourceRecord::new(
            ResourceId::from_stable_label(label),
            kind,
            ResourceLocator::parse(locator).unwrap(),
        );
        let row = Arc::new(ResourceManagementRow::from_record(&record));
        summary.add(&row);
        rows_by_shard[resource_management_shard_index(row.id)].push(row);
    }
    let shards = rows_by_shard
        .into_iter()
        .map(|rows| Arc::new(ResourceManagementShard::from_rows(rows)))
        .collect();
    let generation = Arc::new(ResourceManagementGeneration::from_parts(7, summary, shards));
    let mut scan = generation.scan(ResourceManagementQuery {
        kind: Some(ResourceKind::Scene),
        state: None,
    });

    let first = scan.next_row().unwrap();
    let second = scan.next_row().unwrap();

    assert_eq!(first.primary_locator.as_ref(), "res://scenes/a.scene.toml");
    assert_eq!(second.primary_locator.as_ref(), "res://scenes/z.scene.toml");
    assert!(scan.next_row().is_none());
    assert!(scan.is_complete());
    assert_eq!(scan.total_matching_count(), 2);
}

#[cfg(feature = "profiling")]
fn profiling_generation() -> Arc<ResourceManagementGeneration> {
    let manager = ResourceManager::new();
    for (label, locator, kind) in [
        ("page-a", "res://models/a.glb", ResourceKind::Model),
        ("page-b", "res://models/b.glb", ResourceKind::Model),
        ("page-skip", "res://assets/a.png", ResourceKind::Texture),
    ] {
        manager.register_record(ResourceRecord::new(
            ResourceId::from_stable_label(label),
            kind,
            ResourceLocator::parse(locator).unwrap(),
        ));
    }
    manager.management_generation()
}

#[cfg(feature = "profiling")]
#[test]
fn resource_management_scan_reports_query_local_profile_metrics() {
    let generation = profiling_generation();
    let mut scan = generation.scan(ResourceManagementQuery {
        kind: Some(ResourceKind::Model),
        state: None,
    });

    assert!(scan.next_row().is_some());
    assert!(scan.next_row().is_some());
    assert!(scan.next_row().is_none());

    assert_eq!(
        scan.profile_metrics(),
        ResourceManagementScanProfileMetrics {
            shard_candidate_checks: 192,
            filtered_rows_skipped: 1,
            rows_emitted: 2,
        }
    );
}

#[cfg(feature = "profiling")]
#[test]
fn resource_management_scan_clone_preserves_cursor_and_profile_metrics() {
    let generation = profiling_generation();
    let mut scan = generation.scan(ResourceManagementQuery {
        kind: Some(ResourceKind::Model),
        state: None,
    });

    let first = scan.next_row().expect("the first model row must exist");
    let mut cloned = scan.clone();

    assert_eq!(cloned.profile_metrics(), scan.profile_metrics());
    let cloned_next = cloned.next_row();
    let scan_next = scan.next_row();
    assert_eq!(cloned_next, scan_next);
    assert_ne!(cloned_next, Some(first));
    assert_eq!(cloned.next_row(), scan.next_row());
    assert_eq!(cloned.profile_metrics(), scan.profile_metrics());
}

#[cfg(feature = "profiling")]
#[test]
fn resource_management_page_reports_query_local_profile_metrics() {
    let generation = profiling_generation();

    let (page, metrics) = generation.profiled_page(
        ResourceManagementQuery {
            kind: Some(ResourceKind::Model),
            state: None,
        },
        1,
        1,
    );

    assert_eq!(page.total_matching_count, 2);
    assert_eq!(page.rows.len(), 1);
    assert_eq!(
        metrics,
        ResourceManagementPageProfileMetrics {
            shard_candidate_checks: 128,
            filtered_rows_skipped: 1,
            candidate_rows: 2,
            rows_returned: 1,
        }
    );
}
