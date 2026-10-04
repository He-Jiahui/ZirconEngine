use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const CANDIDATES_PER_SAMPLE: usize = 4;

#[test]
fn log_directory_candidates_reserve_the_fixed_upper_bound() {
    let source = include_str!("../platform.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(4)"));
    assert!(!implementation.contains("let mut candidates = Vec::new()"));
}

#[test]
fn log_directory_candidate_order_still_places_the_unity_fallback_last_for_local_first() {
    let source = include_str!("../platform.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let current = implementation
        .find("source: \"current-directory\"")
        .expect("current directory candidate");
    let unity = implementation
        .find("source: \"unity-compatible-user-log-directory\"")
        .expect("unity-compatible candidate");
    assert!(current < unity);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830bs_runtime_log_directory_capacity_p95() {
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME371_LOG_DIRECTORY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} candidates_per_sample={CANDIDATES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..256 {
        let mut candidates = if optimized {
            Vec::with_capacity(CANDIDATES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..CANDIDATES_PER_SAMPLE {
            candidates.push(index);
        }
        checksum ^= candidates.len();
    }
    std::hint::black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
