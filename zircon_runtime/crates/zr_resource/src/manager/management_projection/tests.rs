use super::*;

use crate::management_generation::RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS;
use crate::{
    ResourceDiagnostic, ResourceId, ResourceKind, ResourceLocator, ResourceManagementQuery,
    ResourceManager, ResourceRecord, ResourceState,
};

mod profile;

fn record(locator: &str, kind: ResourceKind, state: ResourceState) -> ResourceRecord {
    let locator = ResourceLocator::parse(locator).unwrap();
    ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator).with_state(state)
}

fn assert_generation_matches_records(
    generation: &Arc<ResourceManagementGeneration>,
    expected: &HashMap<ResourceId, ResourceRecord>,
) {
    let mut scan = generation.scan(ResourceManagementQuery::default());
    let mut previous = None::<Arc<ResourceManagementRow>>;
    let mut scanned = 0usize;
    while let Some(row) = scan.next_row() {
        if let Some(previous) = &previous {
            assert!(resource_management_row_order(previous, &row).is_le());
        }
        let record = expected.get(&row.id).expect("scanned row remains expected");
        assert!(
            record
                .primary_locator
                .matches_display(row.primary_locator.as_ref())
        );
        assert_eq!(row.revision, record.revision);
        assert!(Arc::ptr_eq(
            &row,
            &generation.row_by_id(row.id).expect("ID index row")
        ));
        assert!(Arc::ptr_eq(
            &row,
            &generation
                .row_by_locator(row.primary_locator.as_ref())
                .expect("locator index row")
        ));
        previous = Some(row);
        scanned += 1;
    }
    assert_eq!(scanned, expected.len());
    assert!(scan.is_complete());
}

#[test]
fn stable_resource_management_poll_reuses_the_exact_generation() {
    let manager = ResourceManager::new();
    let first = record(
        "res://models/a.glb",
        ResourceKind::Model,
        ResourceState::Ready,
    );

    manager.register_record(first.clone()).unwrap();
    let published = manager.management_generation();
    manager.register_record(first).unwrap();
    let stable = manager.management_generation();

    assert!(Arc::ptr_eq(&published, &stable));
    assert_eq!(stable.summary().total_count(), 1);
}

#[test]
fn non_projected_record_changes_reuse_the_exact_management_generation() {
    let manager = ResourceManager::new();
    let first = record(
        "res://models/a.glb",
        ResourceKind::Model,
        ResourceState::Ready,
    )
    .with_diagnostics(vec![ResourceDiagnostic::error("first diagnostic")]);

    manager.register_record(first.clone()).unwrap();
    let published = manager.management_generation();
    manager
        .register_record(
            first.with_diagnostics(vec![ResourceDiagnostic::error("second diagnostic")]),
        )
        .unwrap();
    let stable = manager.management_generation();

    assert!(Arc::ptr_eq(&published, &stable));
}

#[test]
fn non_projected_record_updates_compare_before_building_a_management_row() {
    let source = include_str!("../management_projection.rs");
    let update = source
        .split("for record in records")
        .nth(1)
        .and_then(|source| source.split("let mut shards").next())
        .expect("resource-management record update loop");
    let comparison = update
        .find("resource_management_row_matches_record")
        .expect("projection update must compare the existing row first");
    let row_build = update
        .find("ResourceManagementRow::from_record_reusing_identity")
        .expect("projection update must retain row materialization for real changes");

    assert!(comparison < row_build);
}

#[test]
fn management_generation_separates_ordered_pages_from_identity_shards() {
    let source = include_str!("../../management_generation.rs");

    assert!(source.contains("RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS"));
    assert!(source.contains("RESOURCE_MANAGEMENT_ID_SHARD_COUNT"));
    assert!(source.contains("ordered_pages"));
    assert!(source.contains("id_shards"));
    assert!(!source.contains("BinaryHeap"));
    assert!(!source.contains("ResourceManagementMergeCandidate"));
    assert!(!source.contains("ResourceManagementShard"));
}

#[test]
fn management_projection_plans_ordered_storage_and_accelerators_independently() {
    let source = include_str!("../management_projection.rs");

    assert!(source.contains("ResourceManagementOrderedStrategy"));
    assert!(source.contains("ResourceManagementIndexStrategy"));
    assert!(source.contains("ordered_strategy"));
    assert!(source.contains("id_index_strategy"));
    assert!(source.contains("locator_index_strategy"));
    assert!(source.contains("apply_projection_plan"));
    assert!(!source.contains("rows.into_values().collect()"));
    assert!(!source.contains("ResourceManagementShard::from_rows"));
}

#[test]
fn resource_management_sharding_uses_independent_randomized_hash_authorities() {
    let source = include_str!("../../management_generation.rs");

    assert!(source.contains("ResourceManagementHashAuthority"));
    assert!(source.contains("RandomState"));
    assert!(source.contains("id_shard_hasher"));
    assert!(source.contains("id_map_hasher"));
    assert!(source.contains("locator_shard_hasher"));
    assert!(!source.contains("DefaultHasher"));
    assert!(!source.contains("RESOURCE_MANAGEMENT_ID_HASH_SEED"));
    assert!(!source.contains("ResourceManagementIdHasher"));
}

#[test]
fn mixed_structural_planning_keeps_accelerators_sparse() {
    let records = (0..4_096usize)
        .map(|index| {
            record(
                &format!("res://planner/{index:04}.asset"),
                ResourceKind::Data,
                ResourceState::Ready,
            )
        })
        .collect::<Vec<_>>();
    let mut projection = ResourceManagementProjection::default();
    projection.apply_delta([], records.iter());
    let generation = projection.generation();
    let mut changes = HashMap::<ResourceId, ProjectedResourceChange>::new();
    for record in &records[..1_024] {
        let previous = generation.row_by_id(record.id).unwrap();
        let mut updated = record.clone();
        updated.revision = 2;
        changes.insert(
            record.id,
            ProjectedResourceChange {
                next: Some(Arc::new(
                    ResourceManagementRow::from_record_reusing_identity(
                        &updated,
                        Some(previous.as_ref()),
                    ),
                )),
                previous: Some(previous),
            },
        );
    }
    let renamed_record = &records[3_500];
    let previous = generation.row_by_id(renamed_record.id).unwrap();
    let mut renamed = renamed_record.clone();
    renamed.primary_locator = ResourceLocator::parse("res://planner-renamed/last.asset").unwrap();
    changes.insert(
        renamed.id,
        ProjectedResourceChange {
            next: Some(Arc::new(
                ResourceManagementRow::from_record_reusing_identity(
                    &renamed,
                    Some(previous.as_ref()),
                ),
            )),
            previous: Some(previous),
        },
    );

    let plan = projection_plan(&generation, &changes);
    assert_eq!(
        plan.ordered_strategy,
        ResourceManagementOrderedStrategy::RebalanceRanges
    );
    assert_eq!(
        plan.id_index_strategy,
        ResourceManagementIndexStrategy::Sparse
    );
    assert_eq!(
        plan.locator_index_strategy,
        ResourceManagementIndexStrategy::Sparse
    );
}

#[test]
fn dense_same_key_revision_rebuilds_only_id_values_and_reuses_locator_index() {
    let records = (0..1_024usize)
        .map(|index| {
            record(
                &format!("res://dense-revision/{index:04}.asset"),
                ResourceKind::Data,
                ResourceState::Ready,
            )
        })
        .collect::<Vec<_>>();
    let mut projection = ResourceManagementProjection::default();
    projection.apply_delta([], records.iter());
    let generation = projection.generation();
    let changes = records
        .iter()
        .map(|record| {
            let previous = generation.row_by_id(record.id).unwrap();
            let mut updated = record.clone();
            updated.revision = 2;
            (
                record.id,
                ProjectedResourceChange {
                    next: Some(Arc::new(
                        ResourceManagementRow::from_record_reusing_identity(
                            &updated,
                            Some(previous.as_ref()),
                        ),
                    )),
                    previous: Some(previous),
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let plan = projection_plan(&generation, &changes);
    assert_eq!(
        plan.ordered_strategy,
        ResourceManagementOrderedStrategy::ReplacePages
    );
    assert_eq!(
        plan.id_index_strategy,
        ResourceManagementIndexStrategy::Rebuild
    );
    assert_eq!(
        plan.locator_index_strategy,
        ResourceManagementIndexStrategy::Reuse
    );
}

#[test]
fn dense_rename_rebuilds_changed_id_values_and_locator_membership() {
    let records = (0..512usize)
        .map(|index| {
            record(
                &format!("res://dense-rename/{index:04}.asset"),
                ResourceKind::Data,
                ResourceState::Ready,
            )
        })
        .collect::<Vec<_>>();
    let mut projection = ResourceManagementProjection::default();
    projection.apply_delta([], records.iter());
    let generation = projection.generation();
    let changes = records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            let previous = generation.row_by_id(record.id).unwrap();
            let mut renamed = record.clone();
            renamed.primary_locator =
                ResourceLocator::parse(&format!("res://dense-renamed/{index:04}.asset")).unwrap();
            (
                record.id,
                ProjectedResourceChange {
                    next: Some(Arc::new(
                        ResourceManagementRow::from_record_reusing_identity(
                            &renamed,
                            Some(previous.as_ref()),
                        ),
                    )),
                    previous: Some(previous),
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let plan = projection_plan(&generation, &changes);
    assert_eq!(
        plan.id_index_strategy,
        ResourceManagementIndexStrategy::Rebuild
    );
    assert_eq!(
        plan.locator_index_strategy,
        ResourceManagementIndexStrategy::Rebuild
    );
}

#[test]
fn dense_remove_rebuilds_empty_membership_indexes() {
    let records = (0..512usize)
        .map(|index| {
            record(
                &format!("res://dense-remove/{index:04}.asset"),
                ResourceKind::Data,
                ResourceState::Ready,
            )
        })
        .collect::<Vec<_>>();
    let mut projection = ResourceManagementProjection::default();
    projection.apply_delta([], records.iter());
    let generation = projection.generation();
    let changes = records
        .iter()
        .map(|record| {
            let previous = generation.row_by_id(record.id).unwrap();
            (
                record.id,
                ProjectedResourceChange {
                    next: None,
                    previous: Some(previous),
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let plan = projection_plan(&generation, &changes);
    assert_eq!(
        plan.id_index_strategy,
        ResourceManagementIndexStrategy::Rebuild
    );
    assert_eq!(
        plan.locator_index_strategy,
        ResourceManagementIndexStrategy::Rebuild
    );
}

#[test]
fn structural_insertions_are_binned_once_before_range_rebalancing() {
    let source = include_str!("../management_projection.rs");
    let rebalance = source
        .split("fn rebalance_ordered_page_ranges")
        .nth(1)
        .and_then(|source| source.split("fn sparse_id_shards").next())
        .expect("structural range rebalance implementation");

    assert!(rebalance.contains("structural_insertions_by_range"));
    assert!(rebalance.contains("partition_point"));
    assert!(rebalance.contains("!order_key_is_unchanged(change)"));
    assert!(!rebalance.contains("rows.extend(changes.values()"));
}

#[test]
fn resource_management_generation_pages_in_locator_order_and_filters_without_full_records() {
    let manager = ResourceManager::new();
    manager
        .register_record(record(
            "res://textures/z.png",
            ResourceKind::Texture,
            ResourceState::Ready,
        ))
        .unwrap();
    manager
        .register_record(record(
            "res://models/b.glb",
            ResourceKind::Model,
            ResourceState::Error,
        ))
        .unwrap();
    manager
        .register_record(record(
            "res://models/a.glb",
            ResourceKind::Model,
            ResourceState::Ready,
        ))
        .unwrap();

    let generation = manager.management_generation();
    let page = generation.page(
        ResourceManagementQuery {
            kind: Some(ResourceKind::Model),
            state: None,
        },
        0,
        16,
    );

    assert_eq!(page.generation, generation.identity());
    assert_eq!(page.total_matching_count, 2);
    assert_eq!(
        page.rows
            .iter()
            .map(|row| row.primary_locator.as_ref())
            .collect::<Vec<_>>(),
        vec!["res://models/a.glb", "res://models/b.glb"]
    );
    assert_eq!(
        generation.summary().kind(ResourceKind::Model).error_count,
        1
    );
}

#[test]
fn resource_management_generation_tracks_state_rename_and_remove() {
    let manager = ResourceManager::new();
    let original = ResourceLocator::parse("res://models/a.glb").unwrap();
    let id = ResourceId::from_locator(&original);
    manager
        .register_record(ResourceRecord::new(
            id,
            ResourceKind::Model,
            original.clone(),
        ))
        .unwrap();
    let first_publication_count = manager
        .management_generation()
        .diagnostics()
        .publication_count;

    manager
        .rename(
            &original,
            ResourceLocator::parse("res://models/renamed.glb").unwrap(),
        )
        .unwrap();
    let renamed = manager.management_generation();
    assert!(renamed.diagnostics().publication_count > first_publication_count);
    assert!(renamed.row_by_locator("res://models/a.glb").is_none());
    assert_eq!(
        renamed.row_by_id(id).unwrap().primary_locator.as_ref(),
        "res://models/renamed.glb"
    );

    manager
        .remove_by_locator(&ResourceLocator::parse("res://models/renamed.glb").unwrap())
        .unwrap();
    let removed = manager.management_generation();
    assert!(removed.row_by_id(id).is_none());
    assert_eq!(removed.summary().total_count(), 0);
}

#[test]
fn resource_management_locator_lookup_uses_the_generation_accelerator() {
    let source = include_str!("../../management_generation.rs");
    let lookup = source
        .split("pub fn row_by_locator")
        .nth(1)
        .and_then(|source| source.split("pub fn scan").next())
        .expect("resource-management locator lookup");

    assert!(lookup.contains("self.locator_shard_index(locator)"));
    assert!(lookup.contains("locator_shards"));
    assert!(!lookup.contains(".iter()"));
    assert!(!lookup.contains("find_map"));
}

#[test]
fn projected_record_update_reuses_the_exact_locator_index() {
    let manager = ResourceManager::new();
    let first = record(
        "res://models/indexed.glb",
        ResourceKind::Model,
        ResourceState::Ready,
    )
    .with_source_hash("source-v1");

    manager.register_lazy_record(first.clone()).unwrap();
    let published = manager.management_generation();
    manager
        .register_lazy_record(first.with_source_hash("source-v2"))
        .unwrap();
    let updated = manager.management_generation();

    assert!(!Arc::ptr_eq(&published, &updated));
    assert!(Arc::ptr_eq(
        &published.locator_shards_arc(),
        &updated.locator_shards_arc()
    ));
}

#[test]
fn locator_index_does_not_assume_the_id_was_derived_from_the_locator() {
    let manager = ResourceManager::new();
    let locator = ResourceLocator::parse("res://models/catalog-owned.glb").unwrap();
    let id = ResourceId::from_stable_label("asset-catalog-owned-id");
    manager
        .register_record(ResourceRecord::new(
            id,
            ResourceKind::Model,
            locator.clone(),
        ))
        .unwrap();

    assert_eq!(
        manager
            .management_generation()
            .row_by_locator(&locator.to_string())
            .expect("locator accelerator row")
            .id,
        id
    );
}

#[test]
fn locator_index_applies_batch_swaps_after_removing_all_old_mappings() {
    let first = record(
        "res://models/swap-a.glb",
        ResourceKind::Model,
        ResourceState::Ready,
    );
    let second = record(
        "res://models/swap-b.glb",
        ResourceKind::Model,
        ResourceState::Ready,
    );
    let mut projection = ResourceManagementProjection::default();
    projection.apply_delta([], [&first, &second]);

    let mut swapped_first = first.clone();
    swapped_first.primary_locator = second.primary_locator.clone();
    let mut swapped_second = second.clone();
    swapped_second.primary_locator = first.primary_locator.clone();
    projection.apply_delta([], [&swapped_first, &swapped_second]);
    let generation = projection.generation();

    assert_eq!(
        generation
            .row_by_locator(&first.primary_locator.to_string())
            .expect("first locator remains indexed")
            .id,
        second.id
    );
    assert_eq!(
        generation
            .row_by_locator(&second.primary_locator.to_string())
            .expect("second locator remains indexed")
            .id,
        first.id
    );
}

#[test]
fn locator_index_replaces_an_old_id_at_the_same_locator() {
    let locator = ResourceLocator::parse("res://models/replaced.glb").unwrap();
    let first = ResourceRecord::new(
        ResourceId::from_stable_label("first-catalog-id"),
        ResourceKind::Model,
        locator.clone(),
    );
    let replacement = ResourceRecord::new(
        ResourceId::from_stable_label("replacement-catalog-id"),
        ResourceKind::Model,
        locator.clone(),
    );
    let mut projection = ResourceManagementProjection::default();
    projection.apply_delta([], [&first]);
    projection.apply_delta([first.id], [&replacement]);
    let generation = projection.generation();

    assert!(generation.row_by_id(first.id).is_none());
    assert_eq!(
        generation
            .row_by_locator(&locator.to_string())
            .expect("replacement locator row")
            .id,
        replacement.id
    );
}

#[test]
fn locator_addition_reuses_every_unaffected_locator_shard() {
    let mut projection = ResourceManagementProjection::default();
    let first = record(
        "res://models/structural-sharing-a.glb",
        ResourceKind::Model,
        ResourceState::Ready,
    );
    projection.apply_delta([], [&first]);
    let published = projection.generation();
    let first_shard = published.locator_shard_index(&first.primary_locator.to_string());
    let second = (0usize..)
        .map(|index| {
            record(
                &format!("res://models/structural-sharing-b-{index}.glb"),
                ResourceKind::Model,
                ResourceState::Ready,
            )
        })
        .find(|record| {
            published.locator_shard_index(&record.primary_locator.to_string()) != first_shard
        })
        .expect("a locator in another shard");
    let changed_shard = published.locator_shard_index(&second.primary_locator.to_string());

    projection.apply_delta([], [&second]);
    let updated = projection.generation();
    for index in 0..updated.locator_shards().len() {
        assert_eq!(
            Arc::ptr_eq(
                &published.locator_shards()[index],
                &updated.locator_shards()[index]
            ),
            index != changed_shard,
            "unexpected locator shard identity at index {index}"
        );
    }
}

#[test]
fn structural_changes_split_then_merge_a_page_without_losing_identity() {
    let mut projection = ResourceManagementProjection::default();
    let records = (0..256usize)
        .map(|index| {
            record(
                &format!("res://page-boundary/{index:04}.asset"),
                ResourceKind::Data,
                ResourceState::Ready,
            )
        })
        .collect::<Vec<_>>();
    projection.apply_delta([], records.iter());
    assert_eq!(projection.generation().ordered_pages().len(), 1);

    let inserted = record(
        "res://page-boundary/0128-extra.asset",
        ResourceKind::Data,
        ResourceState::Ready,
    );
    projection.apply_delta([], [&inserted]);
    let split = projection.generation();
    assert_eq!(split.ordered_pages().len(), 2);
    assert!(
        split
            .ordered_pages()
            .iter()
            .all(|page| page.len() <= RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS)
    );
    assert!(split.row_by_id(inserted.id).is_some());

    projection.apply_delta([inserted.id], std::iter::empty());
    let merged = projection.generation();
    assert_eq!(merged.ordered_pages().len(), 1);
    assert!(merged.row_by_id(inserted.id).is_none());
    let expected = records
        .into_iter()
        .map(|record| (record.id, record))
        .collect::<HashMap<_, _>>();
    assert_generation_matches_records(&merged, &expected);
}

#[test]
fn disjoint_structural_ranges_match_a_canonical_oracle_and_reuse_middle_pages() {
    let mut projection = ResourceManagementProjection::default();
    let records = (0..4_096usize)
        .map(|index| {
            record(
                &format!("res://disjoint/{index:04}.asset"),
                ResourceKind::Data,
                ResourceState::Ready,
            )
        })
        .collect::<Vec<_>>();
    projection.apply_delta([], records.iter());
    let published = projection.generation();
    let mut expected = records
        .iter()
        .cloned()
        .map(|record| (record.id, record))
        .collect::<HashMap<_, _>>();

    let removed_ids = records[320..580]
        .iter()
        .chain(&records[3_200..3_460])
        .map(|record| record.id)
        .collect::<Vec<_>>();
    for id in &removed_ids {
        expected.remove(id);
    }
    let additions = (0..260usize)
        .flat_map(|index| {
            [
                record(
                    &format!("res://disjoint/0450-a-{index:04}.asset"),
                    ResourceKind::Data,
                    ResourceState::Ready,
                ),
                record(
                    &format!("res://disjoint/3350-z-{index:04}.asset"),
                    ResourceKind::Data,
                    ResourceState::Ready,
                ),
            ]
        })
        .collect::<Vec<_>>();
    for record in &additions {
        expected.insert(record.id, record.clone());
    }

    projection.apply_delta(removed_ids, additions.iter());
    let updated = projection.generation();
    assert_generation_matches_records(&updated, &expected);
    let reused_pages = published
        .ordered_pages()
        .iter()
        .filter(|before| {
            updated
                .ordered_pages()
                .iter()
                .any(|after| Arc::ptr_eq(before, after))
        })
        .count();
    assert!(reused_pages >= 6, "expected disjoint middle-page sharing");
}

#[test]
fn duplicate_remove_and_upsert_inputs_publish_only_the_final_batch_state() {
    let original = record(
        "res://duplicates/original.asset",
        ResourceKind::Data,
        ResourceState::Ready,
    );
    let mut projection = ResourceManagementProjection::default();
    projection.apply_delta([], [&original]);
    let published = projection.generation();

    projection.apply_delta([original.id, original.id], [&original, &original]);
    assert!(Arc::ptr_eq(&published, &projection.generation()));

    let mut first_update = original.clone();
    first_update.revision = 2;
    let mut final_update = original.clone();
    final_update.revision = 3;
    projection.apply_delta([original.id, original.id], [&first_update, &final_update]);
    let updated = projection.generation();
    assert_eq!(updated.row_by_id(original.id).unwrap().revision, 3);
    assert_eq!(updated.summary().total_count(), 1);
}

#[test]
fn lazy_registration_batch_publishes_one_generation_for_many_records() {
    let manager = ResourceManager::new();
    manager
        .register_lazy_records([
            record(
                "res://models/a.glb",
                ResourceKind::Model,
                ResourceState::Ready,
            ),
            record(
                "res://models/b.glb",
                ResourceKind::Model,
                ResourceState::Ready,
            ),
            record(
                "res://textures/a.png",
                ResourceKind::Texture,
                ResourceState::Ready,
            ),
        ])
        .unwrap();

    let generation = manager.management_generation();
    assert_eq!(generation.diagnostics().publication_count, 1);
    assert_eq!(generation.summary().total_count(), 3);
}
