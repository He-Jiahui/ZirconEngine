use std::collections::HashMap;

use super::super::declarations::{
    VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy, VisibilityHistoryEntry,
    VisibilityHistorySnapshot,
};

pub(crate) fn build_bvh_update_plan(
    current_instances: &[VisibilityHistoryEntry],
    previous: Option<&VisibilityHistorySnapshot>,
) -> VisibilityBvhUpdatePlan {
    let Some(previous) = previous else {
        return VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::FullRebuild,
            inserted_stable_instance_keys: current_instances
                .iter()
                .map(|entry| entry.stable_instance_key)
                .collect(),
            updated_stable_instance_keys: Vec::new(),
            removed_stable_instance_keys: Vec::new(),
        };
    };

    if previous.instances.is_empty() {
        return VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::FullRebuild,
            inserted_stable_instance_keys: current_instances
                .iter()
                .map(|entry| entry.stable_instance_key)
                .collect(),
            updated_stable_instance_keys: Vec::new(),
            removed_stable_instance_keys: Vec::new(),
        };
    }

    let previous_by_stable_instance_key = previous
        .instances
        .iter()
        .map(|entry| (entry.stable_instance_key, entry))
        .collect::<HashMap<_, _>>();
    let current_by_stable_instance_key = current_instances
        .iter()
        .map(|entry| (entry.stable_instance_key, entry))
        .collect::<HashMap<_, _>>();
    let inserted_stable_instance_keys = current_instances
        .iter()
        .filter(|entry| !previous_by_stable_instance_key.contains_key(&entry.stable_instance_key))
        .map(|entry| entry.stable_instance_key)
        .collect::<Vec<_>>();
    let updated_stable_instance_keys = current_instances
        .iter()
        .filter(|entry| {
            previous_by_stable_instance_key
                .get(&entry.stable_instance_key)
                .is_some_and(|old| **old != **entry)
        })
        .map(|entry| entry.stable_instance_key)
        .collect::<Vec<_>>();
    let removed_stable_instance_keys = previous
        .instances
        .iter()
        .filter(|entry| !current_by_stable_instance_key.contains_key(&entry.stable_instance_key))
        .map(|entry| entry.stable_instance_key)
        .collect::<Vec<_>>();

    VisibilityBvhUpdatePlan {
        strategy: VisibilityBvhUpdateStrategy::Incremental,
        inserted_stable_instance_keys,
        updated_stable_instance_keys,
        removed_stable_instance_keys,
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeMap, HashMap};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::core::framework::render::RenderLayerSet;
    use crate::core::framework::scene::Mobility;
    use crate::core::math::Real;
    use crate::core::resource::ResourceId;

    use super::super::super::declarations::{VisibilityBatchKey, VisibilityBounds};
    use super::*;

    const ENTRY_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 17;

    fn history_entry(stable_instance_key: u64, radius: Real) -> VisibilityHistoryEntry {
        VisibilityHistoryEntry {
            entity: stable_instance_key,
            stable_instance_key,
            key: VisibilityBatchKey {
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
                material_id: ResourceId::from_stable_label("bvh-update-material"),
                model_id: ResourceId::from_stable_label("bvh-update-model"),
                mobility: Mobility::Static,
            },
            bounds: VisibilityBounds {
                radius,
                ..VisibilityBounds::default()
            },
        }
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn legacy_index_checksum(current: &[(u64, u64)], previous: &[(u64, u64)]) -> usize {
        let previous_by_key = previous.iter().copied().collect::<BTreeMap<_, _>>();
        let current_by_key = current.iter().copied().collect::<BTreeMap<_, _>>();
        let inserted = current
            .iter()
            .filter(|(key, _)| !previous_by_key.contains_key(key))
            .count();
        let updated = current
            .iter()
            .filter(|(key, value)| previous_by_key.get(key).is_some_and(|old| old != value))
            .count();
        let removed = previous
            .iter()
            .filter(|(key, _)| !current_by_key.contains_key(key))
            .count();
        inserted + updated + removed
    }

    fn optimized_index_checksum(current: &[(u64, u64)], previous: &[(u64, u64)]) -> usize {
        let previous_by_key = previous.iter().copied().collect::<HashMap<_, _>>();
        let current_by_key = current.iter().copied().collect::<HashMap<_, _>>();
        let inserted = current
            .iter()
            .filter(|(key, _)| !previous_by_key.contains_key(key))
            .count();
        let updated = current
            .iter()
            .filter(|(key, value)| previous_by_key.get(key).is_some_and(|old| old != value))
            .count();
        let removed = previous
            .iter()
            .filter(|(key, _)| !current_by_key.contains_key(key))
            .count();
        inserted + updated + removed
    }

    #[test]
    fn optimization_batch_20260826o_runtime09b_hash_indexes_preserve_delta_order() {
        let previous = VisibilityHistorySnapshot {
            instances: vec![
                history_entry(30, 1.0),
                history_entry(10, 1.0),
                history_entry(20, 1.0),
            ],
            ..VisibilityHistorySnapshot::default()
        };
        let current = vec![
            history_entry(20, 2.0),
            history_entry(40, 1.0),
            history_entry(10, 1.0),
        ];

        let plan = build_bvh_update_plan(&current, Some(&previous));

        assert_eq!(plan.strategy, VisibilityBvhUpdateStrategy::Incremental);
        assert_eq!(plan.inserted_stable_instance_keys, vec![40]);
        assert_eq!(plan.updated_stable_instance_keys, vec![20]);
        assert_eq!(plan.removed_stable_instance_keys, vec![30]);
    }

    #[test]
    fn optimization_batch_20260826o_runtime09b_bvh_update_uses_hash_indexes() {
        let source = include_str!("build_bvh_update_plan.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashMap;"));
        assert_eq!(production.matches("collect::<HashMap<_, _>>()").count(), 2);
        assert!(!production.contains("BTreeMap"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826o_runtime09b_bvh_update_hash_index_performance_evidence() {
        let previous = (0..ENTRY_COUNT as u64)
            .map(|key| (key, key))
            .collect::<Vec<_>>();
        let current = (0..ENTRY_COUNT as u64)
            .map(|key| (key, key.wrapping_add((key % 8 == 0) as u64)))
            .collect::<Vec<_>>();
        assert_eq!(legacy_index_checksum(&current, &previous), ENTRY_COUNT / 8);
        assert_eq!(
            optimized_index_checksum(&current, &previous),
            ENTRY_COUNT / 8
        );

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_index_checksum(
                    black_box(&current),
                    black_box(&previous),
                ));
                legacy_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(optimized_index_checksum(
                    black_box(&current),
                    black_box(&previous),
                ));
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_index_checksum(
                    black_box(&current),
                    black_box(&previous),
                ));
                optimized_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(legacy_index_checksum(
                    black_box(&current),
                    black_box(&previous),
                ));
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME09B_BVH_UPDATE_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} \
             ordered_index_admissions={} hash_index_admissions={} lookup_probes={} \
             legacy_p95_ns={} optimized_p95_ns={}",
            ENTRY_COUNT * 2,
            ENTRY_COUNT * 2,
            ENTRY_COUNT * 3,
            legacy_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
            "hash-index P95 {:?} exceeded 60% of ordered-index P95 {:?}",
            optimized_p95,
            legacy_p95,
        );
    }
}
