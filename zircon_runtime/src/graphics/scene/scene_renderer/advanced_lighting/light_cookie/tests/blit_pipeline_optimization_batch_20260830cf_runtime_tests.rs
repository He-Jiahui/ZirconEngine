use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const ENTRIES_PER_SAMPLE: usize = 512;

#[test]
fn light_cookie_draw_collection_reserves_entry_capacity() {
    let source = include_str!("../blit_pipeline.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("light cookie blit implementation");

    assert!(implementation.contains("let mut draws = Vec::with_capacity(entries.len())"));
    assert!(implementation.contains("for entry in entries"));
    assert!(implementation.contains("draws.push((viewport, bind_group))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cf_runtime_light_cookie_capacity_p95() {
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
        "RUNTIME384_LIGHT_COOKIE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entries_per_sample={ENTRIES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut draws = if use_capacity {
            Vec::with_capacity(ENTRIES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for entry in 0..ENTRIES_PER_SAMPLE {
            if entry % 5 != 0 {
                draws.push((entry % 16, entry / 16));
            }
        }
        checksum ^= draws.len();
    }
    std::hint::black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
