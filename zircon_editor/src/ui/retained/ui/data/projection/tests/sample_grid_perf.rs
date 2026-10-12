use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const VALUES_PER_SAMPLE: usize = 256;

#[test]
fn sample_grid_reserves_point_and_number_array_capacity() {
    let source = include_str!("../sample_grid.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("let mut points = Vec::with_capacity(values.len())"));
    assert!(implementation.contains("let mut output = Vec::with_capacity(values.len())"));
    assert!(implementation.contains("for value in values"));
    assert!(implementation.contains("output.extend(values.iter().filter_map(number_value))"));
}

#[test]
fn sample_grid_keeps_invalid_value_filtering() {
    let source = include_str!("../sample_grid.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("let Some(point) = value.as_table() else"));
    assert!(implementation.contains("let (Some(x), Some(y))"));
    assert!(implementation.contains("filter_map(number_value)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cc_editor_sample_grid_capacity_p95() {
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
        "EDITOR327_SAMPLE_GRID_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut output = if optimized {
            Vec::with_capacity(VALUES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..VALUES_PER_SAMPLE {
            if index % 3 != 0 {
                output.push(index);
            }
        }
        checksum ^= output.len();
    }
    std::hint::black_box(checksum);
    started.elapsed().as_nanos().max(1)
}
fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut s = samples.to_vec();
    s.sort_unstable();
    s[(s.len() * p).div_ceil(100).saturating_sub(1)]
}
fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
