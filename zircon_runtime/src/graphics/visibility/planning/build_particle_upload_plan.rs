use super::super::declarations::{VisibilityHistorySnapshot, VisibilityParticleUploadPlan};
use crate::core::framework::scene::EntityId;

fn sorted_difference(source: &[EntityId], comparison: &[EntityId]) -> Vec<EntityId> {
    let mut difference = Vec::new();
    let mut comparison_index = 0;
    for entity in source.iter().copied() {
        while comparison
            .get(comparison_index)
            .is_some_and(|candidate| *candidate < entity)
        {
            comparison_index += 1;
        }
        if comparison.get(comparison_index).copied() != Some(entity) {
            difference.push(entity);
        }
    }
    difference
}

pub(crate) fn build_particle_upload_plan(
    current: &VisibilityHistorySnapshot,
    previous: Option<&VisibilityHistorySnapshot>,
) -> VisibilityParticleUploadPlan {
    let emitter_entities = current.particle_emitters.clone();
    let Some(previous) = previous else {
        return VisibilityParticleUploadPlan {
            emitter_entities: emitter_entities.clone(),
            dirty_emitters: emitter_entities,
            removed_emitters: Vec::new(),
        };
    };

    if previous.particle_emitters.is_empty() {
        return VisibilityParticleUploadPlan {
            emitter_entities: emitter_entities.clone(),
            dirty_emitters: emitter_entities,
            removed_emitters: Vec::new(),
        };
    }

    let dirty_emitters = sorted_difference(&emitter_entities, &previous.particle_emitters);
    let removed_emitters = sorted_difference(&previous.particle_emitters, &emitter_entities);

    VisibilityParticleUploadPlan {
        emitter_entities,
        dirty_emitters,
        removed_emitters,
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const EMITTER_COUNT: usize = 100_000;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn legacy_delta_checksum(current: &[u64], previous: &[u64]) -> usize {
        let previous_set = previous.iter().copied().collect::<BTreeSet<_>>();
        let current_set = current.iter().copied().collect::<BTreeSet<_>>();
        let dirty = current
            .iter()
            .filter(|entity| !previous_set.contains(entity))
            .count();
        let removed = previous
            .iter()
            .filter(|entity| !current_set.contains(entity))
            .count();
        dirty + removed
    }

    fn optimized_delta_checksum(current: &[u64], previous: &[u64]) -> usize {
        sorted_difference(current, previous).len() + sorted_difference(previous, current).len()
    }

    #[test]
    fn optimization_batch_20260826q_runtime09b_linear_particle_difference_preserves_plan_order() {
        let previous = VisibilityHistorySnapshot {
            particle_emitters: vec![1, 5, 8, 21],
            ..VisibilityHistorySnapshot::default()
        };
        let current = VisibilityHistorySnapshot {
            particle_emitters: vec![2, 5, 8, 13],
            ..VisibilityHistorySnapshot::default()
        };

        let plan = build_particle_upload_plan(&current, Some(&previous));

        assert_eq!(plan.emitter_entities, vec![2, 5, 8, 13]);
        assert_eq!(plan.dirty_emitters, vec![2, 13]);
        assert_eq!(plan.removed_emitters, vec![1, 21]);
    }

    #[test]
    fn optimization_batch_20260826q_runtime09b_particle_upload_uses_linear_difference() {
        let source = include_str!("build_particle_upload_plan.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("fn sorted_difference("));
        assert_eq!(production.matches("sorted_difference(").count(), 3);
        assert!(!production.contains("BTreeSet"));
        assert!(!production.contains("contains("));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826q_runtime09b_particle_upload_linear_difference_performance_evidence()
     {
        let previous = (0..EMITTER_COUNT as u64)
            .map(|index| index * 2)
            .collect::<Vec<_>>();
        let current = (0..EMITTER_COUNT as u64)
            .map(|index| index * 2 + u64::from(index % 8 == 0))
            .collect::<Vec<_>>();
        let expected = EMITTER_COUNT / 4;
        assert_eq!(legacy_delta_checksum(&current, &previous), expected);
        assert_eq!(optimized_delta_checksum(&current, &previous), expected);

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_delta_checksum(
                    black_box(&current),
                    black_box(&previous),
                ));
                legacy_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(optimized_delta_checksum(
                    black_box(&current),
                    black_box(&previous),
                ));
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_delta_checksum(
                    black_box(&current),
                    black_box(&previous),
                ));
                optimized_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(legacy_delta_checksum(
                    black_box(&current),
                    black_box(&previous),
                ));
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME09B_PARTICLE_UPLOAD_LINEAR_DIFFERENCE_BENCH_V1 current_emitters={EMITTER_COUNT} \
             previous_emitters={EMITTER_COUNT} ordered_index_admissions={} membership_probes={} \
             linear_input_visits={} legacy_p95_ns={} optimized_p95_ns={}",
            EMITTER_COUNT * 2,
            EMITTER_COUNT * 2,
            EMITTER_COUNT * 4,
            legacy_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
            "linear-difference P95 {:?} exceeded 60% of tree-difference P95 {:?}",
            optimized_p95,
            legacy_p95,
        );
    }
}
