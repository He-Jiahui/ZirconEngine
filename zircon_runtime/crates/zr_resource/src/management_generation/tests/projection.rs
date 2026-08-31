use super::super::*;
use crate::{ResourceLocator, ResourceManager};

#[test]
fn resource_management_scan_cursor_yields_matching_rows_once_in_stable_order() {
    let rows = [
        ("scan-z", "res://scenes/z.scene.toml", ResourceKind::Scene),
        ("scan-a", "res://scenes/a.scene.toml", ResourceKind::Scene),
        ("scan-m", "res://meshes/m.mesh.toml", ResourceKind::Mesh),
    ]
    .into_iter()
    .map(|(label, locator, kind)| {
        let record = ResourceRecord::new(
            ResourceId::from_stable_label(label),
            kind,
            ResourceLocator::parse(locator).unwrap(),
        );
        Arc::new(ResourceManagementRow::from_record(&record))
    })
    .collect();
    let generation = Arc::new(ResourceManagementGeneration::from_rows(7, rows));
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

#[test]
fn sparse_revision_update_reuses_unaffected_pages_and_indexes() {
    let manager = ResourceManager::new();
    let records = (0..1_024)
        .map(|index| {
            let locator = ResourceLocator::parse(&format!("res://models/{index:04}.glb")).unwrap();
            ResourceRecord::new(
                ResourceId::from_locator(&locator),
                ResourceKind::Model,
                locator,
            )
            .with_source_hash("source-v1")
            .with_state(ResourceState::Ready)
        })
        .collect::<Vec<_>>();
    manager.register_lazy_records(records.clone()).unwrap();
    let published = manager.management_generation();
    let published_row = published.row_by_id(records[511].id).unwrap();

    manager
        .register_lazy_record(records[511].clone().with_source_hash("source-v2"))
        .unwrap();
    let updated = manager.management_generation();
    let updated_row = updated.row_by_id(records[511].id).unwrap();

    assert!(Arc::ptr_eq(
        &published_row.primary_locator,
        &updated_row.primary_locator
    ));

    assert_eq!(
        published.ordered_pages().len(),
        updated.ordered_pages().len()
    );
    let reused_pages = published
        .ordered_pages()
        .iter()
        .zip(updated.ordered_pages())
        .filter(|(before, after)| Arc::ptr_eq(before, after))
        .count();
    assert!(reused_pages >= published.ordered_pages().len().saturating_sub(3));
    assert!(reused_pages > 0);

    let changed_id_shard = published.id_shard_index(records[511].id);
    for index in 0..published.id_shards().len() {
        assert_eq!(
            Arc::ptr_eq(&published.id_shards()[index], &updated.id_shards()[index]),
            index != changed_id_shard,
            "unexpected ID shard identity at index {index}"
        );
    }
    assert!(
        published
            .locator_shards()
            .iter()
            .zip(updated.locator_shards())
            .all(|(before, after)| Arc::ptr_eq(before, after))
    );
}

#[test]
fn dense_revision_rebuild_preserves_canonical_scan_and_lookup_identity() {
    let manager = ResourceManager::new();
    let records = (0..2_048)
        .rev()
        .map(|index| {
            let locator = ResourceLocator::parse(&format!("res://dense/{index:04}.asset")).unwrap();
            ResourceRecord::new(
                ResourceId::from_locator(&locator),
                ResourceKind::Data,
                locator,
            )
            .with_source_hash("source-v1")
            .with_state(ResourceState::Ready)
        })
        .collect::<Vec<_>>();
    manager.register_lazy_records(records.clone()).unwrap();
    let published = manager.management_generation();
    manager
        .register_lazy_records(
            records
                .iter()
                .cloned()
                .map(|record| record.with_source_hash("source-v2")),
        )
        .unwrap();

    let generation = manager.management_generation();
    assert!(Arc::ptr_eq(
        &published.locator_shards_arc(),
        &generation.locator_shards_arc()
    ));
    let mut scan = generation.scan(ResourceManagementQuery::default());
    let mut previous = None::<Arc<ResourceManagementRow>>;
    let mut scanned = 0usize;
    while let Some(row) = scan.next_row() {
        if let Some(previous) = &previous {
            assert!(resource_management_row_order(previous, &row).is_le());
        }
        assert!(Arc::ptr_eq(
            &row,
            &generation.row_by_id(row.id).expect("ID index row")
        ));
        assert!(Arc::ptr_eq(
            &row,
            &generation
                .row_by_locator(&row.primary_locator)
                .expect("locator index row")
        ));
        previous = Some(row);
        scanned += 1;
    }
    assert_eq!(scanned, records.len());
    assert!(scan.is_complete());
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
            shard_candidate_checks: 1,
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
            shard_candidate_checks: 1,
            filtered_rows_skipped: 1,
            candidate_rows: 2,
            rows_returned: 1,
        }
    );
}
