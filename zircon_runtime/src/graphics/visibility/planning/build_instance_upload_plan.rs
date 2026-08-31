use std::collections::HashSet;

use crate::core::framework::scene::Mobility;

use super::super::declarations::{
    VisibilityBvhInstance, VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy,
    VisibilityInstanceUploadPlan,
};

pub(crate) fn build_instance_upload_plan(
    bvh_instances: &[VisibilityBvhInstance],
    bvh_update_plan: &VisibilityBvhUpdatePlan,
) -> VisibilityInstanceUploadPlan {
    let static_instance_keys = bvh_instances
        .iter()
        .filter(|instance| instance.key.mobility == Mobility::Static)
        .map(|instance| instance.stable_instance_key)
        .collect::<Vec<_>>();
    let dynamic_instance_keys = bvh_instances
        .iter()
        .filter(|instance| instance.key.mobility == Mobility::Dynamic)
        .map(|instance| instance.stable_instance_key)
        .collect::<Vec<_>>();
    let dynamic_instance_key_set = dynamic_instance_keys
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let dirty_dynamic_set = match bvh_update_plan.strategy {
        VisibilityBvhUpdateStrategy::FullRebuild => dynamic_instance_key_set,
        VisibilityBvhUpdateStrategy::Incremental => bvh_update_plan
            .inserted_stable_instance_keys
            .iter()
            .chain(bvh_update_plan.updated_stable_instance_keys.iter())
            .copied()
            .filter(|key| dynamic_instance_key_set.contains(key))
            .collect(),
    };
    let dirty_dynamic_instance_keys = dynamic_instance_keys
        .iter()
        .copied()
        .filter(|key| dirty_dynamic_set.contains(key))
        .collect::<Vec<_>>();

    VisibilityInstanceUploadPlan {
        static_instance_keys,
        dynamic_instance_keys,
        dirty_dynamic_instance_keys,
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::core::framework::render::RenderLayerSet;
    use crate::core::math::Vec3;
    use crate::core::resource::ResourceId;

    use super::super::super::declarations::{VisibilityBatchKey, VisibilityBounds};
    use super::*;

    const DYNAMIC_KEY_COUNT: usize = 8_192;
    const DIRTY_LOOKUP_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn instance(stable_instance_key: u64, mobility: Mobility) -> VisibilityBvhInstance {
        VisibilityBvhInstance {
            entity: stable_instance_key,
            stable_instance_key,
            key: VisibilityBatchKey {
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                material_id: ResourceId::from_stable_label("tests/material"),
                model_id: ResourceId::from_stable_label("tests/model"),
                mobility,
            },
            bounds: VisibilityBounds {
                center: Vec3::new(stable_instance_key as f32, 0.0, 0.0),
                radius: 1.0,
            },
        }
    }

    fn dynamic_keys() -> Vec<u64> {
        (0..DYNAMIC_KEY_COUNT).map(|index| index as u64).collect()
    }

    fn dirty_lookups(dynamic_keys: &[u64]) -> Vec<u64> {
        (0..DIRTY_LOOKUP_COUNT)
            .map(|index| dynamic_keys[(index * 4_099) % dynamic_keys.len()])
            .collect()
    }

    fn ordered_membership_count(dynamic_keys: &[u64], lookups: &[u64]) -> usize {
        let dynamic = dynamic_keys.iter().copied().collect::<BTreeSet<_>>();
        lookups.iter().filter(|key| dynamic.contains(*key)).count()
    }

    fn hash_membership_count(dynamic_keys: &[u64], lookups: &[u64]) -> usize {
        let dynamic = dynamic_keys.iter().copied().collect::<HashSet<_>>();
        lookups.iter().filter(|key| dynamic.contains(*key)).count()
    }

    #[test]
    fn optimization_batch_20260826v_runtime09b_hash_upload_membership_preserves_input_order() {
        let instances = vec![
            instance(5, Mobility::Static),
            instance(30, Mobility::Dynamic),
            instance(10, Mobility::Dynamic),
            instance(20, Mobility::Dynamic),
        ];
        let update = VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::Incremental,
            inserted_stable_instance_keys: vec![20, 30, 999],
            updated_stable_instance_keys: vec![10],
            ..VisibilityBvhUpdatePlan::default()
        };

        let plan = build_instance_upload_plan(&instances, &update);

        assert_eq!(plan.static_instance_keys, vec![5]);
        assert_eq!(plan.dynamic_instance_keys, vec![30, 10, 20]);
        assert_eq!(plan.dirty_dynamic_instance_keys, vec![30, 10, 20]);
    }

    #[test]
    fn optimization_batch_20260826v_runtime09b_instance_upload_uses_hash_membership() {
        let source = include_str!("build_instance_upload_plan.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("collect::<HashSet<_>>()"));
        assert!(production.contains("dynamic_instance_key_set.contains(key)"));
        assert!(production.contains("dirty_dynamic_set.contains(key)"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826v_runtime09b_instance_upload_hash_membership_performance_evidence()
     {
        let dynamic_keys = dynamic_keys();
        let lookups = dirty_lookups(&dynamic_keys);
        assert_eq!(
            ordered_membership_count(&dynamic_keys, &lookups),
            hash_membership_count(&dynamic_keys, &lookups)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_membership_count(
                    black_box(&dynamic_keys),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_membership_count(
                    black_box(&dynamic_keys),
                    black_box(&lookups),
                ));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_membership_count(
                    black_box(&dynamic_keys),
                    black_box(&lookups),
                ));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_membership_count(
                    black_box(&dynamic_keys),
                    black_box(&lookups),
                ));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "RUNTIME09B_INSTANCE_UPLOAD_HASH_MEMBERSHIP_BENCH_V1 dynamic_keys={DYNAMIC_KEY_COUNT} \
             lookups={DIRTY_LOOKUP_COUNT} ordered_lookup_class=log_n \
             hash_lookup_class=average_constant ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-membership P95 {:?} exceeded 60% of ordered-membership P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
