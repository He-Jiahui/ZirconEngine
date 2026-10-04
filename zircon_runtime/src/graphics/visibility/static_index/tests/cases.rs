use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::render::RenderLayerSet;
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::resource::ResourceId;
use crate::graphics::visibility::{VisibilityBatchKey, VisibilityBvhUpdateStrategy};

#[test]
fn visibility_static_index_incremental_update_matches_full_rebuild_queries() {
    let first_frame = vec![
        instance(1, Vec3::new(0.0, 0.0, 0.0), 1.0),
        instance(2, Vec3::new(40.0, 0.0, 0.0), 1.0),
    ];
    let second_frame = vec![
        instance(2, Vec3::new(48.0, 0.0, 0.0), 1.0),
        instance(3, Vec3::new(0.0, 20.0, 0.0), 1.0),
    ];
    let plan = VisibilityBvhUpdatePlan {
        strategy: VisibilityBvhUpdateStrategy::Incremental,
        inserted_stable_instance_keys: vec![3],
        updated_stable_instance_keys: vec![2],
        removed_stable_instance_keys: vec![1],
    };
    let mut incremental = VisibilityStaticIndex::new(16.0);
    incremental.rebuild(&first_frame);

    let update_report = incremental.apply_update_plan(&second_frame, &plan);

    let mut full = VisibilityStaticIndex::new(16.0);
    let rebuild_report = full.rebuild(&second_frame);
    let full_scene_query = VisibilityBounds {
        center: Vec3::new(24.0, 10.0, 0.0),
        radius: 64.0,
    };
    let removed_entity_query = VisibilityBounds {
        center: Vec3::ZERO,
        radius: 2.0,
    };

    assert_eq!(update_report.full_rebuild_count, 1);
    assert_eq!(update_report.incremental_update_count, 1);
    assert_eq!(update_report.frame_full_rebuild_count, 0);
    assert_eq!(update_report.frame_incremental_update_count, 1);
    assert_eq!(update_report.inserted_count, 1);
    assert_eq!(update_report.updated_count, 1);
    assert_eq!(update_report.removed_count, 1);
    assert_eq!(update_report.indexed_entity_count, 2);
    assert_eq!(rebuild_report.full_rebuild_count, 1);
    assert_eq!(
        incremental.query_bounds(full_scene_query),
        full.query_bounds(full_scene_query)
    );
    assert_eq!(
        incremental.query_bounds(removed_entity_query),
        Vec::<EntityId>::new()
    );
}

#[test]
fn visibility_static_index_incremental_projection_keeps_last_duplicate_instance() {
    let mut index = VisibilityStaticIndex::new(16.0);
    index.rebuild(&[instance(1, Vec3::new(-32.0, 0.0, 0.0), 1.0)]);
    let current = [
        instance(1, Vec3::ZERO, 1.0),
        instance(1, Vec3::new(64.0, 0.0, 0.0), 1.0),
    ];

    index.apply_update_plan(
        &current,
        &VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::Incremental,
            inserted_stable_instance_keys: Vec::new(),
            updated_stable_instance_keys: vec![1],
            removed_stable_instance_keys: Vec::new(),
        },
    );

    assert_eq!(
        index.query_bounds(VisibilityBounds {
            center: Vec3::ZERO,
            radius: 2.0,
        }),
        Vec::<u64>::new(),
    );
    assert_eq!(
        index.query_bounds(VisibilityBounds {
            center: Vec3::new(64.0, 0.0, 0.0),
            radius: 2.0,
        }),
        vec![1],
    );
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260827_runtime09b_dirty_proportional_static_index_evidence() {
    const INSTANCE_COUNT: usize = 65_536;
    const CHANGED_KEY_COUNT: usize = 128;
    const SAMPLE_COUNT: usize = 21;
    let instances = (0..INSTANCE_COUNT as u64)
        .map(|key| (key, key.wrapping_mul(17).rotate_left(7)))
        .collect::<Vec<_>>();
    let changed_keys = (0..CHANGED_KEY_COUNT as u64)
        .map(|index| {
            if index % 8 == 0 {
                INSTANCE_COUNT as u64 + index
            } else {
                index * (INSTANCE_COUNT as u64 / CHANGED_KEY_COUNT as u64)
            }
        })
        .collect::<Vec<_>>();

    let legacy = |instances: &[(u64, u64)], changed_keys: &[u64]| {
        let by_key = instances.iter().copied().collect::<BTreeMap<_, _>>();
        changed_keys.iter().fold(0_u64, |checksum, key| {
            checksum.wrapping_add(
                by_key
                    .get(key)
                    .copied()
                    .unwrap_or_else(|| key.rotate_left(11)),
            )
        })
    };
    let optimized = |instances: &[(u64, u64)], changed_keys: &[u64]| {
        let mut by_changed_key = HashMap::with_capacity(changed_keys.len());
        for key in changed_keys {
            by_changed_key.insert(*key, None);
        }
        for (key, value) in instances {
            if let Some(slot) = by_changed_key.get_mut(key) {
                *slot = Some(*value);
            }
        }
        changed_keys.iter().fold(0_u64, |checksum, key| {
            checksum.wrapping_add(
                by_changed_key
                    .get(key)
                    .copied()
                    .flatten()
                    .unwrap_or_else(|| key.rotate_left(11)),
            )
        })
    };
    assert_eq!(
        legacy(&instances, &changed_keys),
        optimized(&instances, &changed_keys)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy(black_box(&instances), black_box(&changed_keys)));
            legacy_samples.push(started.elapsed().as_nanos());
            let started = Instant::now();
            black_box(optimized(black_box(&instances), black_box(&changed_keys)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized(black_box(&instances), black_box(&changed_keys)));
            optimized_samples.push(started.elapsed().as_nanos());
            let started = Instant::now();
            black_box(legacy(black_box(&instances), black_box(&changed_keys)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }
    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[(SAMPLE_COUNT - 1) * 95 / 100];
    let optimized_p95 = optimized_samples[(SAMPLE_COUNT - 1) * 95 / 100];
    println!(
        "RUNTIME09B_DIRTY_PROPORTIONAL_STATIC_INDEX_BENCH_V1 instances={INSTANCE_COUNT} changed_keys={CHANGED_KEY_COUNT} legacy_index_entries={INSTANCE_COUNT} optimized_index_entries={CHANGED_KEY_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=6000"
    );
    assert!(CHANGED_KEY_COUNT * 100 <= INSTANCE_COUNT * 5);
    assert!(optimized_p95 * 100 <= legacy_p95 * 60);
}

#[test]
fn visibility_static_index_full_rebuild_strategy_replaces_existing_rows() {
    let mut index = VisibilityStaticIndex::new(16.0);
    index.rebuild(&[instance(1, Vec3::ZERO, 1.0)]);
    let report = index.apply_update_plan(
        &[instance(2, Vec3::new(32.0, 0.0, 0.0), 1.0)],
        &VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::FullRebuild,
            inserted_stable_instance_keys: vec![2],
            updated_stable_instance_keys: Vec::new(),
            removed_stable_instance_keys: Vec::new(),
        },
    );

    assert_eq!(report.full_rebuild_count, 2);
    assert_eq!(report.incremental_update_count, 0);
    assert_eq!(report.frame_full_rebuild_count, 1);
    assert_eq!(report.frame_incremental_update_count, 0);
    assert_eq!(index.report().indexed_entity_count, 1);
    assert_eq!(
        index.query_bounds(VisibilityBounds {
            center: Vec3::new(32.0, 0.0, 0.0),
            radius: 2.0,
        }),
        vec![2]
    );
}

#[test]
fn visibility_static_index_clone_shares_persistent_storage_until_a_mutation() {
    let mut index = VisibilityStaticIndex::new(16.0);
    index.rebuild(&[instance(1, Vec3::ZERO, 1.0)]);
    let snapshot = index.clone();

    assert!(Arc::ptr_eq(&index.entries, &snapshot.entries));
    assert!(Arc::ptr_eq(&index.cells, &snapshot.cells));
    assert!(Arc::ptr_eq(
        &index.overflow_instance_keys,
        &snapshot.overflow_instance_keys,
    ));

    index.rebuild(&[instance(2, Vec3::new(32.0, 0.0, 0.0), 1.0)]);

    assert_eq!(
        snapshot.query_bounds(VisibilityBounds {
            center: Vec3::ZERO,
            radius: 2.0,
        }),
        vec![1]
    );
    assert_eq!(
        index.query_bounds(VisibilityBounds {
            center: Vec3::new(32.0, 0.0, 0.0),
            radius: 2.0,
        }),
        vec![2]
    );
}

#[test]
fn visibility_static_index_bounded_query_refuses_extreme_cell_volume() {
    let mut index = VisibilityStaticIndex::new(16.0);
    index.rebuild(&[instance(1, Vec3::ZERO, 1.0)]);

    assert_eq!(
        index.query_bounds_with_stats_limited(
            VisibilityBounds {
                center: Vec3::ZERO,
                radius: f32::MAX,
            },
            4_096,
        ),
        None,
    );
}

#[test]
fn visibility_static_index_keeps_extreme_bounds_in_conservative_overflow() {
    let mut index = VisibilityStaticIndex::new(16.0);
    let report = index.rebuild(&[instance(1, Vec3::ZERO, f32::MAX)]);

    let query = index
        .query_bounds_with_stats_limited(
            VisibilityBounds {
                center: Vec3::new(1.0, 0.0, 0.0),
                radius: 1.0,
            },
            MAX_CELLS_PER_INDEXED_INSTANCE,
        )
        .expect("small query stays inside the cell budget");

    assert_eq!(report.indexed_entity_count, 1);
    assert_eq!(report.occupied_cell_count, 0);
    assert_eq!(query.stable_instance_keys, vec![1]);
}

#[test]
fn visibility_static_index_large_internal_query_keeps_all_entries_conservative() {
    let mut index = VisibilityStaticIndex::new(16.0);
    index.rebuild(&[
        instance(1, Vec3::ZERO, 1.0),
        instance(2, Vec3::new(64.0, 0.0, 0.0), 1.0),
    ]);

    assert_eq!(
        index.query_bounds(VisibilityBounds {
            center: Vec3::ZERO,
            radius: f32::MAX,
        }),
        vec![1, 2],
    );
}

#[test]
fn visibility_static_index_query_normalizes_overlapping_cell_memberships() {
    let mut index = VisibilityStaticIndex::new(16.0);
    index.rebuild(&[
        instance(1, Vec3::ZERO, 20.0),
        instance(2, Vec3::new(64.0, 0.0, 0.0), 1.0),
    ]);

    let query = index
        .query_bounds_with_stats_limited(
            VisibilityBounds {
                center: Vec3::new(32.0, 0.0, 0.0),
                radius: 40.0,
            },
            MAX_CELLS_PER_INDEXED_INSTANCE,
        )
        .expect("bounded query should enumerate its cells");

    assert_eq!(query.stable_instance_keys, vec![1, 2]);
}

#[test]
fn optimization_batch_runtime213_static_index_query_uses_vec_normalization() {
    let source = include_str!("../mod.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("static index production source");

    assert!(production.contains("fn collect_query_keys"));
    assert!(production.contains("stable_instance_keys.sort_unstable();"));
    assert!(production.contains("stable_instance_keys.dedup();"));
    assert!(!production.contains("collect::<BTreeSet<_>>()"));
}

fn instance(entity: EntityId, center: Vec3, radius: Real) -> VisibilityBvhInstance {
    VisibilityBvhInstance {
        entity,
        stable_instance_key: entity,
        key: VisibilityBatchKey {
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            material_id: ResourceId::from_stable_label("tests/material"),
            model_id: ResourceId::from_stable_label("tests/model"),
            mobility: Mobility::Static,
        },
        bounds: VisibilityBounds { center, radius },
    }
}
