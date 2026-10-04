use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const BATCHES_PER_SAMPLE: usize = 64;
const SOURCES_PER_BATCH: usize = 4_096;

#[test]
fn optimization_batch_jl_runtime651_identity_changes_reserve_source_upper_bound() {
    let source = include_str!("../../projected_inventory.rs");

    assert!(source.contains("let mut identity_changes = Vec::with_capacity(sources.len());"));
    assert!(!source.contains("let mut identity_changes = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_jl_runtime651_identity_change_capacity_bench() {
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure(false));
            reserved_samples.push(measure(true));
        } else {
            reserved_samples.push(measure(true));
            unreserved_samples.push(measure(false));
        }
    }

    let unreserved_p50_ns = percentile(&unreserved_samples, 50);
    let reserved_p50_ns = percentile(&reserved_samples, 50);
    let unreserved_p95_ns = percentile(&unreserved_samples, 95);
    let reserved_p95_ns = percentile(&reserved_samples, 95);
    println!(
        "RUNTIME651_IDENTITY_CHANGE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} sources_per_batch={SOURCES_PER_BATCH} \
unreserved_p50_ns={unreserved_p50_ns} reserved_p50_ns={reserved_p50_ns} \
unreserved_p95_ns={unreserved_p95_ns} reserved_p95_ns={reserved_p95_ns} \
unreserved_raw_ns={} reserved_raw_ns={}",
        sample_csv(&unreserved_samples),
        sample_csv(&reserved_samples),
    );

    assert!(reserved_p95_ns <= unreserved_p95_ns * 80 / 100);
}

#[derive(Clone, Copy)]
struct IdentityChangeFixture([usize; 8]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for batch in 0..BATCHES_PER_SAMPLE {
        let mut changes = if reserve {
            Vec::with_capacity(SOURCES_PER_BATCH)
        } else {
            Vec::new()
        };
        for source in 0..SOURCES_PER_BATCH {
            changes.push(IdentityChangeFixture([black_box(batch ^ source); 8]));
        }
        checksum ^= changes[SOURCES_PER_BATCH - 1].0[0] ^ changes.len();
        black_box(&changes);
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
