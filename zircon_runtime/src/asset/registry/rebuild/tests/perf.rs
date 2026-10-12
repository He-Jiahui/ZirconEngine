use std::hint::black_box;
use std::time::Instant;

const META_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jh_runtime647_reserves_reminted_path_output() {
    let source = include_str!("../../rebuild.rs");
    let normalize = source
        .split("pub(super) fn normalize_duplicate_guids")
        .nth(1)
        .expect("duplicate GUID normalization remains present")
        .split("fn normalize_duplicate_guid_document")
        .next()
        .expect("duplicate GUID normalization remains bounded");

    assert!(normalize.contains("Vec::with_capacity(metas.len())"));
    assert!(!normalize.contains("let mut reminted_paths = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jh_runtime647_reminted_path_capacity_benchmark() {
    for _ in 0..4 {
        black_box(measure_paths(false));
        black_box(measure_paths(true));
    }
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_paths(false));
            reserved_samples.push(measure_paths(true));
        } else {
            reserved_samples.push(measure_paths(true));
            unreserved_samples.push(measure_paths(false));
        }
    }
    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let reserved_p95 = percentile(&reserved_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(reserved_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME647_REMINTED_PATH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} meta_count={META_COUNT} unreserved_ns={} reserved_ns={} unreserved_p95_ns={unreserved_p95} reserved_p95_ns={reserved_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&reserved_samples),
    );
    assert!(reserved_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_paths(reserved: bool) -> u128 {
    let started = Instant::now();
    let mut paths = if reserved {
        Vec::with_capacity(META_COUNT)
    } else {
        Vec::new()
    };
    for index in 0..META_COUNT {
        if index % 2 == 0 {
            paths.push(black_box(index));
        }
    }
    black_box(paths);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
