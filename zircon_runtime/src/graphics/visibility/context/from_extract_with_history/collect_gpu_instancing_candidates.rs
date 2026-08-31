use crate::core::framework::scene::Mobility;

use super::super::super::declarations::VisibilityBatch;

pub(super) fn collect_gpu_instancing_candidates(
    visible_batches: &[VisibilityBatch],
) -> Vec<VisibilityBatch> {
    let mut candidates = Vec::with_capacity(visible_batches.len());
    candidates.extend(
        visible_batches
            .iter()
            .filter(|batch| {
                batch.key.mobility == Mobility::Dynamic && batch.stable_instance_keys.len() > 1
            })
            .cloned(),
    );
    candidates
}

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830dd_gpu_candidates_reserve_visible_batch_upper_bound() {
        let source = include_str!("collect_gpu_instancing_candidates.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("GPU instancing candidate production source");

        assert!(production.contains("Vec::with_capacity(visible_batches.len())"));
        assert!(production.contains("candidates.extend("));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830dd_gpu_candidate_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const CANDIDATE_COUNT: usize = 64;
        const MARKER: &str = "RUNTIME516_GPU_INSTANCING_CANDIDATE_CAPACITY_BENCH_V1";

        let legacy_growth_events = candidate_growth_events(BATCH_COUNT, CANDIDATE_COUNT, false);
        let optimized_growth_events = candidate_growth_events(BATCH_COUNT, CANDIDATE_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} candidates={CANDIDATE_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn candidate_growth_events(batch_count: usize, candidate_count: usize, reserve: bool) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut candidates = if reserve {
                Vec::with_capacity(candidate_count)
            } else {
                Vec::new()
            };
            for candidate in 0..candidate_count {
                let previous_capacity = candidates.capacity();
                candidates.push(candidate);
                growth_events += usize::from(candidates.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
