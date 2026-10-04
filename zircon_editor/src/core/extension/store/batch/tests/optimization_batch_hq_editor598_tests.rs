use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_hq_editor598_capability_storage_is_sorted_deduplicated_and_shared() {
    let batch = ContributionBatch::default().with_required_capabilities([
        "plugin.sample.zeta",
        "plugin.sample.alpha",
        "plugin.sample.zeta",
    ]);
    assert_eq!(
        batch.required_capabilities(),
        &[
            "plugin.sample.alpha".to_string(),
            "plugin.sample.zeta".to_string()
        ]
    );

    let cloned = batch.clone();
    assert!(Arc::ptr_eq(
        &batch.required_capabilities,
        &cloned.required_capabilities
    ));
}

#[test]
fn optimization_batch_hq_editor598_store_reuses_shared_capability_storage() {
    let batch_source = include_str!("../../batch.rs");
    let contribution_source = include_str!("../../model/contribution_store.rs");

    assert!(batch_source.contains("required_capabilities: Arc<[String]>"));
    assert!(batch_source.contains("self.required_capabilities = capabilities.into();"));
    assert_eq!(
        contribution_source
            .matches("Arc::clone(&batch.required_capabilities)")
            .count(),
        1
    );
    assert_eq!(
        contribution_source
            .matches("Arc::clone(&replacement_batch.required_capabilities)")
            .count(),
        1
    );
    assert!(!contribution_source.contains("required_capabilities.clone().into()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hq_editor598_capability_share_performance_evidence() {
    let capabilities = (0..4_096)
        .map(|index| {
            format!(
                "plugin.sample.capability.{index:05}.{}",
                "retained-capability-payload".repeat(3)
            )
        })
        .collect::<Vec<_>>();
    let shared: Arc<[String]> = capabilities.clone().into();
    const CLONES_PER_SAMPLE: usize = 32;
    const SAMPLE_PAIRS: usize = 17;
    let measure_legacy = || {
        let started = Instant::now();
        for _ in 0..CLONES_PER_SAMPLE {
            black_box(black_box(&capabilities).clone());
        }
        started.elapsed().as_nanos().max(1)
    };
    let measure_shared = || {
        let started = Instant::now();
        for _ in 0..CLONES_PER_SAMPLE {
            black_box(Arc::clone(black_box(&shared)));
        }
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_shared());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut shared_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            shared_samples.push(measure_shared());
        } else {
            shared_samples.push(measure_shared());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    shared_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let shared_p50 = shared_samples[8];
    let shared_p95 = shared_samples[16];
    println!(
        "EDITOR598_CONTRIBUTION_CAPABILITY_SHARE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 shared_first_pairs=8 capabilities={} clones_per_sample={CLONES_PER_SAMPLE} legacy_p50_ns={} legacy_p95_ns={} shared_p50_ns={} shared_p95_ns={} legacy_string_clones={} shared_string_clones=0 target_ratio_bp=1000",
        capabilities.len(),
        legacy_p50,
        legacy_p95,
        shared_p50,
        shared_p95,
        capabilities.len() * CLONES_PER_SAMPLE,
    );
    assert!(
        shared_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(1_000),
        "shared capability P95 {shared_p95} ns exceeded 10% of legacy {legacy_p95} ns"
    );
}
