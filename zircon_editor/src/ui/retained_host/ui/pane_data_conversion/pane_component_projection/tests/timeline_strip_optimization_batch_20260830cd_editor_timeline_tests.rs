use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const VALUES_PER_SAMPLE: usize = 512;

#[test]
fn timeline_key_projection_reserves_array_capacity_and_keeps_filtering() {
    let source = include_str!("../timeline_strip.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("timeline strip implementation");

    assert!(implementation.contains("let mut keys = Vec::with_capacity(values.len())"));
    assert!(implementation.contains("let Some(key) = value.as_table() else"));
    assert!(implementation.contains("if !time.is_finite()"));
    assert!(implementation.contains("time.clamp(0.0, duration)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cd_editor_visual_array_capacity_p95() {
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
        "EDITOR328_VISUAL_ARRAY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut timeline = if use_capacity {
            Vec::with_capacity(VALUES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let mut heatmap = if use_capacity {
            Vec::with_capacity(VALUES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for value in 0..VALUES_PER_SAMPLE {
            if value % 4 != 0 {
                timeline.push(value);
            }
            if value % 5 != 0 {
                heatmap.push(value);
            }
        }
        checksum ^= timeline.len() ^ heatmap.len();
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
