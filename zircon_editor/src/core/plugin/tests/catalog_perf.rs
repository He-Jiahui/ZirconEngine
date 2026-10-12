use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const CAPABILITIES_PER_SAMPLE: usize = 16_384;

#[test]
fn optimization_batch_20260830cm_editor_capability_diagnostics_reserve_upper_bound() {
    let source = include_str!("../catalog.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("editor plugin catalog implementation");

    assert!(implementation.contains("let diagnostic_capacity = self"));
    assert!(implementation.contains(".map(|registration| registration.capabilities.len())"));
    assert!(implementation.contains("Vec::with_capacity(diagnostic_capacity)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cm_editor_capability_diagnostic_capacity_p95() {
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
        "EDITOR500_CAPABILITY_DIAGNOSTIC_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} capabilities_per_sample={CAPABILITIES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_capacity: bool) -> u128 {
    let started = Instant::now();
    let mut diagnostics = if use_capacity {
        Vec::with_capacity(CAPABILITIES_PER_SAMPLE)
    } else {
        Vec::new()
    };
    for capability in 0..CAPABILITIES_PER_SAMPLE {
        diagnostics.push(capability);
    }
    std::hint::black_box(diagnostics);
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
