use std::sync::{Arc, Barrier};
use std::thread;

use crate::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManagementGeneration, ResourceManager,
    ResourceMutationBatch, ResourceRecord, ResourceState,
};

use super::{ResourceRegistryExportSnapshot, should_scan_management_generation};

#[derive(Debug)]
struct TestPayload;

fn record(index: usize) -> ResourceRecord {
    record_with_kind(index, ResourceKind::Model)
}

fn record_with_kind(index: usize, kind: ResourceKind) -> ResourceRecord {
    record_at(&format!("res://models/{:04}.glb", index), kind)
}

fn record_at(locator_text: &str, kind: ResourceKind) -> ResourceRecord {
    let locator = ResourceLocator::parse(locator_text).expect("valid fixture locator");
    ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator)
}

fn sorted(mut records: Vec<ResourceRecord>) -> Vec<ResourceRecord> {
    records.sort_unstable_by(|left, right| {
        left.primary_locator
            .cmp(&right.primary_locator)
            .then_with(|| left.id.cmp(&right.id))
    });
    records
}

fn generation_with_summary_but_no_rows(
    generation: &ResourceManagementGeneration,
) -> Arc<ResourceManagementGeneration> {
    Arc::new(ResourceManagementGeneration::from_parts(
        generation.diagnostics(),
        generation.summary().clone(),
        generation.hash_authority_arc(),
        Arc::from([]),
        generation.id_shards().to_vec().into(),
        generation.locator_shards_arc(),
    ))
}

#[test]
fn frameworks01_resource_manager_registry_export_orders_ready_records() {
    let manager = ResourceManager::new();
    for index in (0..8).rev() {
        manager
            .register_ready(record(index), TestPayload)
            .expect("fixture registration");
    }
    let records = manager.ready_records_for_kind(ResourceKind::Model);
    let expected_locators = sorted((0..8).rev().map(record).collect())
        .into_iter()
        .map(|record| record.primary_locator)
        .collect::<Vec<_>>();
    let actual_locators = records
        .iter()
        .map(|record| record.primary_locator.clone())
        .collect::<Vec<_>>();

    assert_eq!(actual_locators, expected_locators);
    assert!(
        records
            .iter()
            .all(|record| record.state == ResourceState::Ready && record.revision == 1)
    );
}

#[test]
fn frameworks01_resource_manager_registry_export_uses_a_short_lived_paired_snapshot() {
    let source = include_str!("../registry_export.rs");
    assert!(source.contains("struct ResourceRegistryExportSnapshot"));
    assert!(source.contains("authority.registry.clone()"));
    assert!(source.contains("authority.management.generation()"));
    assert!(!source.contains("let registry = self.lock_registry_read();"));
}

#[test]
fn frameworks01_resource_manager_registry_export_planner_matches_profiled_cost_boundaries() {
    assert!(!should_scan_management_generation(1_024, 1));
    assert!(!should_scan_management_generation(1_024, 256));
    assert!(!should_scan_management_generation(10_000, 1_000));
    assert!(should_scan_management_generation(10_000, 2_500));
    assert!(should_scan_management_generation(32_768, 64));
    assert!(!should_scan_management_generation(32_768, 65));
    assert!(!should_scan_management_generation(32_768, 3_276));
    assert!(should_scan_management_generation(32_768, 3_277));
    assert!(!should_scan_management_generation(100_000, 1_000));
    assert!(should_scan_management_generation(100_000, 100_000));
}

#[test]
fn frameworks01_resource_manager_registry_export_management_scan_matches_registry_export() {
    let manager = ResourceManager::new();
    for index in (0..8).rev() {
        manager
            .register_ready(record(index), TestPayload)
            .expect("fixture registration");
    }

    let snapshot = ResourceRegistryExportSnapshot::capture(&manager);
    let registry_records = snapshot.ready_records_from_registry(ResourceKind::Model, 8);
    let management_records = snapshot
        .ready_records_from_management(ResourceKind::Model, 8)
        .expect("paired management export");

    assert_eq!(management_records, registry_records);
}

#[test]
fn frameworks01_resource_manager_registry_export_paths_share_canonical_cross_scheme_order() {
    const RECORD_COUNT: usize = 4_096;
    const MODEL_COUNT: usize = 1_024;

    let manager = ResourceManager::new();
    let mut batch = ResourceMutationBatch::new();
    for locator in [
        "res://models/special.glb",
        "package://com.zircon.models/special.glb",
        "mem://models/special.glb",
        "lib://models/special.glb",
        "builtin://models/special.glb",
    ] {
        batch = batch.upsert_ready(record_at(locator, ResourceKind::Model), TestPayload);
    }
    for index in 5..MODEL_COUNT {
        batch = batch.upsert_ready(record(index), TestPayload);
    }
    for index in MODEL_COUNT..RECORD_COUNT {
        batch = batch.upsert_ready(record_with_kind(index, ResourceKind::Shader), TestPayload);
    }
    manager.commit(batch).expect("fixture commit");

    assert!(should_scan_management_generation(RECORD_COUNT, MODEL_COUNT));
    let snapshot = ResourceRegistryExportSnapshot::capture(&manager);
    let registry_records = snapshot.ready_records_from_registry(ResourceKind::Model, MODEL_COUNT);
    let management_records = snapshot
        .ready_records_from_management(ResourceKind::Model, MODEL_COUNT)
        .expect("paired management export");
    let display_order = registry_records
        .iter()
        .map(|record| record.primary_locator.to_string())
        .collect::<Vec<_>>();

    assert!(display_order.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(management_records, registry_records);
    assert_eq!(
        snapshot.ready_records_for_kind(ResourceKind::Model),
        management_records
    );
}

#[test]
fn frameworks01_resource_manager_registry_export_snapshot_pairs_matching_generations() {
    let manager = ResourceManager::new();
    manager
        .register_ready(record(7), TestPayload)
        .expect("fixture registration");

    let snapshot = ResourceRegistryExportSnapshot::capture(&manager);
    let management_row = snapshot
        .management
        .row_by_id(record(7).id)
        .expect("management row");
    let registry_record = snapshot
        .registry
        .get(management_row.id)
        .expect("paired registry record");

    assert_eq!(registry_record.kind, management_row.kind);
    assert_eq!(
        registry_record.primary_locator.to_string(),
        management_row.primary_locator.as_ref()
    );
    assert_eq!(registry_record.revision, management_row.revision);
    assert_eq!(registry_record.state, management_row.state);
}

#[test]
fn frameworks01_resource_manager_registry_export_snapshot_stays_atomic_during_commits() {
    const RECORD_COUNT: usize = 64;
    const COMMIT_COUNT: usize = 64;

    let manager = Arc::new(ResourceManager::new());
    for index in 0..RECORD_COUNT {
        manager
            .register_ready(record(index), TestPayload)
            .expect("initial fixture registration");
    }

    let start = Arc::new(Barrier::new(2));
    let writer_manager = Arc::clone(&manager);
    let writer_start = Arc::clone(&start);
    let writer = thread::spawn(move || {
        writer_start.wait();
        for generation in 0..COMMIT_COUNT {
            let mut batch = ResourceMutationBatch::new();
            for index in 0..RECORD_COUNT {
                batch = batch.upsert_ready(
                    record(index).with_source_hash(format!("generation-{generation}")),
                    TestPayload,
                );
            }
            writer_manager.commit(batch).expect("fixture commit");
            thread::yield_now();
        }
    });

    start.wait();
    for _ in 0..COMMIT_COUNT {
        let snapshot = ResourceRegistryExportSnapshot::capture(&manager);
        let records = snapshot
            .ready_records_from_management(ResourceKind::Model, RECORD_COUNT)
            .expect("paired management export");
        assert_eq!(records.len(), RECORD_COUNT);
        let revision = records[0].revision;
        assert!(records.iter().all(|record| record.revision == revision));
        thread::yield_now();
    }
    writer.join().expect("writer thread");
}

#[test]
fn frameworks01_resource_manager_registry_export_falls_back_on_projection_inconsistency() {
    const RECORD_COUNT: usize = 4_096;
    const MODEL_COUNT: usize = 1_024;

    let manager = ResourceManager::new();
    let mut batch = ResourceMutationBatch::new();
    for index in 0..RECORD_COUNT {
        let kind = if index < MODEL_COUNT {
            ResourceKind::Model
        } else {
            ResourceKind::Shader
        };
        batch = batch.upsert_ready(record_with_kind(index, kind), TestPayload);
    }
    manager.commit(batch).expect("fixture commit");

    let mut snapshot = ResourceRegistryExportSnapshot::capture(&manager);
    snapshot.registry.remove_by_id(record(0).id);
    let expected = snapshot.ready_records_from_registry(ResourceKind::Model, MODEL_COUNT);

    assert_eq!(
        snapshot.ready_records_for_kind(ResourceKind::Model),
        expected
    );
}

#[test]
fn frameworks01_resource_manager_registry_export_falls_back_when_projection_scan_is_incomplete() {
    const RECORD_COUNT: usize = 4_096;
    const MODEL_COUNT: usize = 1_024;

    let manager = ResourceManager::new();
    let mut batch = ResourceMutationBatch::new();
    for index in 0..RECORD_COUNT {
        let kind = if index < MODEL_COUNT {
            ResourceKind::Model
        } else {
            ResourceKind::Shader
        };
        batch = batch.upsert_ready(record_with_kind(index, kind), TestPayload);
    }
    manager.commit(batch).expect("fixture commit");

    let mut snapshot = ResourceRegistryExportSnapshot::capture(&manager);
    let expected = snapshot.ready_records_from_registry(ResourceKind::Model, MODEL_COUNT);
    snapshot.management = generation_with_summary_but_no_rows(&snapshot.management);

    assert_eq!(
        snapshot.ready_records_for_kind(ResourceKind::Model),
        expected
    );
}

#[test]
fn frameworks01_resource_manager_registry_export_falls_back_on_projection_metadata_drift() {
    const RECORD_COUNT: usize = 4_096;
    const MODEL_COUNT: usize = 1_024;

    let manager = ResourceManager::new();
    let mut batch = ResourceMutationBatch::new();
    for index in 0..RECORD_COUNT {
        let kind = if index < MODEL_COUNT {
            ResourceKind::Model
        } else {
            ResourceKind::Shader
        };
        batch = batch.upsert_ready(record_with_kind(index, kind), TestPayload);
    }
    manager.commit(batch).expect("fixture commit");

    let mut snapshot = ResourceRegistryExportSnapshot::capture(&manager);
    let mismatched = snapshot
        .registry
        .remove_by_id(record(0).id)
        .expect("fixture registry row")
        .with_state(ResourceState::Error);
    snapshot.registry.insert_unchecked(mismatched);
    let expected = snapshot.ready_records_from_registry(ResourceKind::Model, MODEL_COUNT);

    assert_eq!(
        snapshot.ready_records_for_kind(ResourceKind::Model),
        expected
    );
}
