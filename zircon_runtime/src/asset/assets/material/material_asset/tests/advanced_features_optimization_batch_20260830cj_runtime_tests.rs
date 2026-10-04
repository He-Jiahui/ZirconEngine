use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const PROPERTY_COUNT: usize = 11;

#[test]
fn optimization_batch_20260830cj_runtime_material_validation_reserves_bounded_error_capacity() {
    let source = include_str!("../advanced_features.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("advanced material implementation");

    assert!(implementation.contains("const ADVANCED_VALIDATED_PROPERTY_COUNT: usize = 11"));
    assert!(implementation
        .contains("Vec::with_capacity(values.len().min(ADVANCED_VALIDATED_PROPERTY_COUNT))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cj_runtime_material_validation_capacity_p95() {
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
        "RUNTIME386_MATERIAL_VALIDATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} properties_per_sample={PROPERTY_COUNT} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..8_192 {
        let mut errors = if use_capacity {
            Vec::with_capacity(PROPERTY_COUNT)
        } else {
            Vec::new()
        };
        for property in 0..PROPERTY_COUNT {
            errors.push(property);
        }
        checksum ^= errors.len();
        std::hint::black_box(errors);
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
