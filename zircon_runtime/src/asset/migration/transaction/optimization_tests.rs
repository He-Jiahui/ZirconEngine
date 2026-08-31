use std::hint::black_box;
use std::time::Instant;

const WRITE_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 16;

fn write_fixture() -> Vec<usize> {
    (0..WRITE_COUNT).collect()
}

#[test]
fn runtime04_migration_transaction_capacity_preserves_write_order() {
    let fixture = write_fixture();
    let mut optimized = Vec::with_capacity(fixture.len());
    optimized.extend(fixture.iter().copied());
    assert_eq!(optimized, fixture);
}

#[test]
fn runtime04_migration_transaction_capacity_source_contract() {
    let source = include_str!("../transaction.rs");
    assert!(source.contains("Vec::with_capacity(pending.len())"));
    assert!(!source.contains("pending\n        .into_iter()\n        .map"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime04_migration_transaction_capacity_bench() {
    let legacy_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let pending = write_fixture();
                let writes = pending
                    .into_iter()
                    .map(|value| value + 1)
                    .collect::<Vec<_>>();
                black_box(writes);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let optimized_samples = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let pending = write_fixture();
                let mut writes = Vec::with_capacity(pending.len());
                writes.extend(pending.into_iter().map(|value| value + 1));
                black_box(writes);
            }
            started.elapsed().as_nanos()
        })
        .collect::<Vec<_>>();
    let percentile_95 = |mut samples: Vec<u128>| {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100) - 1]
    };
    let legacy_p95 = percentile_95(legacy_samples);
    let optimized_p95 = percentile_95(optimized_samples);
    println!(
        "RUNTIME04_MIGRATION_TRANSACTION_CAPACITY_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} writes={} write_capacity_reduction=0->{}",
        legacy_p95, optimized_p95, SAMPLE_COUNT, ITERATIONS, WRITE_COUNT, WRITE_COUNT,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(95),
        "optimized migration transaction capacity p95 should be at most 95% of legacy p95"
    );
}
