use std::hint::black_box;
use std::time::Instant;

use super::{
    export_profile_strategy_diagnostics, export_strategy_diagnostic_capacity,
    ExportPackagingStrategy, ExportProfile,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const STRATEGIES_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826er_runtime187_capacity_preserves_duplicate_diagnostics() {
    let profile = ExportProfile::default().with_strategies(std::iter::repeat_n(
        ExportPackagingStrategy::NativeDynamic,
        STRATEGIES_PER_BUILD,
    ));

    let diagnostics = export_profile_strategy_diagnostics(&profile);

    assert_eq!(diagnostics.len(), STRATEGIES_PER_BUILD - 1);
    assert!(diagnostics.capacity() >= STRATEGIES_PER_BUILD - 1);
    assert!(diagnostics.iter().all(|diagnostic| diagnostic
        .contains("strategies must not repeat packaging strategy NativeDynamic")));
    assert_eq!(export_strategy_diagnostic_capacity(0), 0);
    assert_eq!(export_strategy_diagnostic_capacity(1), 0);
    assert_eq!(
        export_strategy_diagnostic_capacity(STRATEGIES_PER_BUILD),
        STRATEGIES_PER_BUILD - 1
    );
}

#[test]
fn optimization_batch_20260826er_runtime187_strategy_diagnostics_reserve_duplicate_upper_bound() {
    let source = include_str!("../export_profile_validation.rs");
    assert!(source.contains("Vec::with_capacity(export_strategy_diagnostic_capacity("));
    assert!(source.contains("profile.strategies.len()"));
    assert!(source.contains("strategy_count.saturating_sub(1)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826er_runtime187_export_strategy_diagnostic_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME187_EXPORT_STRATEGY_DIAGNOSTIC_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} strategies_per_build={STRATEGIES_PER_BUILD} \
diagnostics_per_build={} legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        STRATEGIES_PER_BUILD - 1,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut diagnostics = if reserve {
            Vec::with_capacity(STRATEGIES_PER_BUILD - 1)
        } else {
            Vec::new()
        };
        for _ in 1..STRATEGIES_PER_BUILD {
            diagnostics.push(black_box(String::new()));
        }
        checksum ^= black_box(diagnostics.len() ^ diagnostics.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
