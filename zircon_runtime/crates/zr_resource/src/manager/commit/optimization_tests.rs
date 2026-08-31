use crate::{
    ResourceEventKind, ResourceId, ResourceKind, ResourceLocator, ResourceManager,
    ResourceMutationBatch, ResourceRecord,
};

use super::{MAX_PREFLIGHT_INITIAL_CAPACITY, StagedResources, preflight_initial_capacity};

fn record(id: &str, locator: &str) -> ResourceRecord {
    ResourceRecord::new(
        ResourceId::from_stable_label(id),
        ResourceKind::Model,
        ResourceLocator::parse(locator).expect("valid resource locator"),
    )
}

#[test]
fn runtime04_resource_manager_commit_ordered_staging_preserves_first_touch_sequence() {
    let first = ResourceId::from_stable_label("ordered-staging-first");
    let second = ResourceId::from_stable_label("ordered-staging-second");
    let third = ResourceId::from_stable_label("ordered-staging-third");
    let mut staged = StagedResources::with_capacity(4);

    staged.get_or_insert_with(second, || None).reload_failed = true;
    staged.get_or_insert_with(first, || None);
    staged.get_or_insert_with(second, || panic!("duplicate slot construction"));
    staged.get_or_insert_with(third, || None);

    assert_eq!(
        staged
            .into_entries()
            .into_iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        vec![second, first, third]
    );
}

#[test]
fn runtime04_resource_manager_commit_ordered_staging_source_contract() {
    let source = include_str!("../commit.rs");
    assert!(source.contains("struct StagedResources"));
    assert!(source.contains("index_by_id: HashMap<ResourceId, usize>"));
    assert!(source.contains("entries: Vec<StagedResource>"));
    assert!(source.contains("preflight_initial_capacity(operations.len())"));
    assert!(!source.contains("StagedResources::with_capacity(operation_capacity)"));
    assert!(!source.contains("HashMap::with_capacity(operation_capacity)"));
    assert!(!source.contains("order: usize"));
    assert!(!source.contains("next_order"));
    assert!(!source.contains("staged.into_values()"));
    assert!(!source.contains("staged.sort_by_key(|entry| entry.order)"));
    assert!(!source.contains("staged.sort_unstable_by_key(|entry| entry.order)"));
}

#[test]
fn runtime04_resource_manager_commit_initial_capacity_is_bounded_for_repeated_batches() {
    let operation_count = 100_000;
    assert_eq!(preflight_initial_capacity(0), 0);
    assert_eq!(preflight_initial_capacity(1), 1);
    assert_eq!(
        preflight_initial_capacity(MAX_PREFLIGHT_INITIAL_CAPACITY),
        MAX_PREFLIGHT_INITIAL_CAPACITY
    );
    assert_eq!(
        preflight_initial_capacity(operation_count),
        MAX_PREFLIGHT_INITIAL_CAPACITY
    );

    let staged = StagedResources::with_capacity(preflight_initial_capacity(operation_count));
    assert_eq!(staged.entries.capacity(), MAX_PREFLIGHT_INITIAL_CAPACITY);
    assert!(staged.index_by_id.capacity() < operation_count);
}

#[test]
fn runtime04_resource_manager_commit_publishes_mixed_events_in_first_touch_order() {
    let manager = ResourceManager::new();
    let update = record("ordered-update", "res://models/ordered-update.glb");
    let remove = record("ordered-remove", "res://models/ordered-remove.glb");
    let rename = record("ordered-rename", "res://models/ordered-rename.glb");
    manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_lazy(update.clone())
                .upsert_lazy(remove.clone())
                .upsert_lazy(rename.clone()),
        )
        .expect("initial ordered records");
    let events = manager.subscribe();
    let added = record("ordered-add", "res://models/ordered-add.glb");
    let mut first_update = update.clone().with_source_hash("first");
    first_update
        .diagnostics
        .push(crate::ResourceDiagnostic::error("first update"));
    let final_update = update.clone().with_source_hash("final");
    let renamed_locator =
        ResourceLocator::parse("res://models/ordered-renamed.glb").expect("renamed locator");

    manager
        .commit(
            ResourceMutationBatch::new()
                .upsert_lazy(first_update)
                .upsert_lazy(added.clone())
                .remove(remove.primary_locator.clone())
                .rename(rename.primary_locator.clone(), renamed_locator)
                .upsert_lazy(final_update),
        )
        .expect("mixed ordered commit");

    let published = (0..4)
        .map(|_| events.recv().expect("ordered resource event"))
        .map(|event| (event.id, event.kind))
        .collect::<Vec<_>>();
    assert_eq!(
        published,
        vec![
            (update.id, ResourceEventKind::Updated),
            (added.id, ResourceEventKind::Added),
            (remove.id, ResourceEventKind::Removed),
            (rename.id, ResourceEventKind::Renamed),
        ]
    );
    assert!(events.try_recv().is_err());
}
