use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const BACKENDS_PER_SAMPLE: usize = 32;
const OUTPUTS_PER_BACKEND: usize = 128;

#[test]
fn picking_backend_collection_reserves_ray_output_estimate() {
    let source = include_str!("../pipeline.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("picking pipeline implementation");

    assert!(implementation.contains("input.backends.len().saturating_mul(ray_map.len())"));
    assert!(implementation.contains("Vec::with_capacity(estimated_output_count)"));
    assert!(implementation.contains("for backend in input.backends"));
    assert!(implementation.contains("backend_outputs.extend(backend.collect_hits(&ray_map))"));
    assert!(!implementation.contains(".flat_map(|backend| backend.collect_hits(&ray_map))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830ck_runtime_picking_backend_capacity_p95() {
    let backends = (0..BACKENDS_PER_SAMPLE)
        .map(|backend| {
            (0..OUTPUTS_PER_BACKEND)
                .map(|output| (backend * OUTPUTS_PER_BACKEND + output) as u64)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&backends, false));
            optimized.push(measure(&backends, true));
        } else {
            optimized.push(measure(&backends, true));
            legacy.push(measure(&backends, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!("RUNTIME387_PICKING_BACKEND_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} backends_per_sample={BACKENDS_PER_SAMPLE} outputs_per_backend={OUTPUTS_PER_BACKEND} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(backends: &[Vec<u64>], use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..256 {
        let outputs = if use_capacity {
            let mut outputs = Vec::with_capacity(BACKENDS_PER_SAMPLE * OUTPUTS_PER_BACKEND);
            for backend in black_box(backends) {
                outputs.extend(backend.iter().copied());
            }
            outputs
        } else {
            black_box(backends)
                .iter()
                .flat_map(|backend| backend.iter().copied())
                .collect::<Vec<_>>()
        };
        checksum ^= outputs.len();
        black_box(outputs);
    }
    black_box(checksum);
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
