use std::collections::HashSet;

use crate::core::framework::platform::RuntimeTargetMode;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_selection::PendingFeatureSelection;
use super::super::feature_status_record::FeatureStatus;

pub(super) fn unresolved_feature_ids(
    pending: &mut [(PendingFeatureSelection<'_>, FeatureStatus)],
) -> HashSet<String> {
    pending
        .iter_mut()
        .map(|(active, _)| std::mem::take(&mut active.definition_key))
        .collect::<HashSet<_>>()
}

pub(super) fn mark_unresolved_feature_cycle(
    status: &mut FeatureStatus,
    projection: &RuntimePluginCatalogProjection,
    unresolved_feature_ids: &HashSet<String>,
    target: RuntimeTargetMode,
) {
    if status.is_waiting_for_feature_capability(projection, unresolved_feature_ids, target) {
        status.mark_cycle();
    }
}

#[cfg(test)]
mod performance_tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn optimization_batch_el_unresolved_feature_ids_move_pending_keys() {
        let source = include_str!("cycle.rs");
        let implementation = source
            .split("pub(super) fn unresolved_feature_ids(")
            .nth(1)
            .expect("unresolved feature id projection")
            .split("pub(super) fn mark_unresolved_feature_cycle(")
            .next()
            .expect("bounded unresolved feature id projection");

        assert!(implementation.contains("pending: &mut ["));
        assert!(implementation.contains("std::mem::take(&mut active.definition_key)"));
        assert!(!implementation.contains("active.definition_key.clone()"));
    }

    fn feature_ids(count: usize) -> Vec<String> {
        (0..count)
            .map(|index| {
                format!(
                    "feature.{}.provider.{index:05}",
                    "runtime-plugin-identity-segment".repeat(8)
                )
            })
            .collect()
    }

    #[test]
    #[ignore = "release-only unresolved feature id move benchmark"]
    fn optimization_batch_el_move_unresolved_feature_ids_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const FEATURE_IDS: usize = 4_096;

        fn measure_legacy(ids: Vec<String>) -> u128 {
            let started = Instant::now();
            let unresolved = ids.iter().cloned().collect::<HashSet<_>>();
            black_box(unresolved);
            black_box(ids);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(mut ids: Vec<String>) -> u128 {
            let started = Instant::now();
            let unresolved = ids.iter_mut().map(std::mem::take).collect::<HashSet<_>>();
            black_box(unresolved);
            black_box(ids);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let base = feature_ids(FEATURE_IDS);
        for _ in 0..4 {
            black_box(measure_legacy(base.clone()));
            black_box(measure_optimized(base.clone()));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy(base.clone()));
                optimized_samples.push(measure_optimized(base.clone()));
            } else {
                optimized_samples.push(measure_optimized(base.clone()));
                legacy_samples.push(measure_legacy(base.clone()));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME446_MOVE_UNRESOLVED_FEATURE_IDS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             feature_ids={FEATURE_IDS} key_bytes={} pair_order=alternating_legacy_even \
             legacy_key_allocations={FEATURE_IDS} optimized_key_allocations=0 \
             legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             legacy_raw_ns={} optimized_raw_ns={}",
            base[0].len(),
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
            "moving unresolved feature ids must reduce P95 by at least 25%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
