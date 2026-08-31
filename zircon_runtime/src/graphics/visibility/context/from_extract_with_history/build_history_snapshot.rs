use std::collections::HashSet;

use crate::core::framework::render::RenderFrameExtract;
use crate::core::framework::scene::EntityId;

use super::super::super::declarations::{VisibilityHistoryEntry, VisibilityHistorySnapshot};

fn sorted_unique_particle_emitters(emitters: &[EntityId]) -> Vec<EntityId> {
    let mut particle_emitters = emitters.iter().copied().collect::<HashSet<_>>();
    let mut particle_emitters = particle_emitters.drain().collect::<Vec<_>>();
    particle_emitters.sort_unstable();
    particle_emitters
}

pub(super) fn build_history_snapshot(
    value: &RenderFrameExtract,
    history_entries: Vec<VisibilityHistoryEntry>,
    hybrid_gi_active_probe_ids: Vec<u32>,
    hybrid_gi_requested_probes: Vec<u32>,
    virtual_geometry_visible_cluster_ids: Vec<u32>,
    virtual_geometry_requested_pages: Vec<u32>,
) -> VisibilityHistorySnapshot {
    VisibilityHistorySnapshot {
        instances: history_entries,
        particle_emitters: sorted_unique_particle_emitters(&value.particles.emitters),
        hybrid_gi_active_probe_ids,
        hybrid_gi_requested_probes,
        virtual_geometry_visible_cluster_ids,
        virtual_geometry_requested_pages,
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const EMITTER_COUNT: usize = 100_000;
    const UNIQUE_EMITTER_COUNT: usize = 50_000;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn legacy_particle_emitters(emitters: &[u64]) -> Vec<u64> {
        emitters
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn optimized_particle_emitters(emitters: &[u64]) -> Vec<u64> {
        let mut unique = emitters.iter().copied().collect::<HashSet<_>>();
        let mut unique = unique.drain().collect::<Vec<_>>();
        unique.sort_unstable();
        unique
    }

    #[test]
    fn optimization_batch_20260826p_runtime09b_particle_history_preserves_sorted_unique_ids() {
        assert_eq!(
            sorted_unique_particle_emitters(&[41, 7, 19, 7, 2, 41]),
            vec![2, 7, 19, 41]
        );
    }

    #[test]
    fn optimization_batch_20260826p_runtime09b_particle_history_uses_hash_dedup() {
        let source = include_str!("build_history_snapshot.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("collect::<HashSet<_>>()"));
        assert!(production.contains("particle_emitters.sort_unstable();"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826p_runtime09b_particle_history_hash_dedup_performance_evidence() {
        let emitters = (0..EMITTER_COUNT)
            .map(|index| ((index * 65_521) % UNIQUE_EMITTER_COUNT) as u64)
            .collect::<Vec<_>>();
        let expected = (0..UNIQUE_EMITTER_COUNT as u64).collect::<Vec<_>>();
        assert_eq!(legacy_particle_emitters(&emitters), expected);
        assert_eq!(optimized_particle_emitters(&emitters), expected);

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_particle_emitters(black_box(&emitters)));
                legacy_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(optimized_particle_emitters(black_box(&emitters)));
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_particle_emitters(black_box(&emitters)));
                optimized_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(legacy_particle_emitters(black_box(&emitters)));
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME09B_PARTICLE_HISTORY_HASH_DEDUP_BENCH_V1 emitters={EMITTER_COUNT} \
             unique_emitters={UNIQUE_EMITTER_COUNT} ordered_admissions={EMITTER_COUNT} \
             hash_admissions={EMITTER_COUNT} sorted_values={UNIQUE_EMITTER_COUNT} \
             legacy_p95_ns={} optimized_p95_ns={}",
            legacy_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
            "hash-dedup P95 {:?} exceeded 60% of ordered-dedup P95 {:?}",
            optimized_p95,
            legacy_p95,
        );
    }
}
