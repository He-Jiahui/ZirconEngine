use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_gz_runtime581_texture_probe_selection_preserves_presence() {
    assert!(has_valid_texture_planar_probe([true, false, false]));
    assert!(has_valid_texture_planar_probe([false, true, false]));
    assert!(!has_valid_texture_planar_probe([false, false, false]));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_gz_runtime581_texture_probe_selection_short_circuit_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 8_192;
    const CANDIDATES: usize = 2_048;
    let mut validities = vec![false; CANDIDATES];
    validities[0] = true;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false, &validities, ITERATIONS));
            optimized.push(measure(true, &validities, ITERATIONS));
        } else {
            optimized.push(measure(true, &validities, ITERATIONS));
            legacy.push(measure(false, &validities, ITERATIONS));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME581_TEXTURE_PROBE_SELECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} candidates={CANDIDATES} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
        "texture probe presence short-circuit must improve P95 by at least 50%"
    );
}

fn measure(optimized: bool, validities: &[bool], iterations: usize) -> u128 {
    let started = Instant::now();
    let mut selected = 0_u64;
    for _ in 0..iterations {
        let result = if optimized {
            has_valid_texture_planar_probe(black_box(validities).iter().copied())
        } else {
            black_box(validities)
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, is_valid)| *is_valid)
                .min_by_key(|(probe_id, _)| *probe_id)
                .is_some()
        };
        selected += u64::from(result);
    }
    black_box(selected);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
