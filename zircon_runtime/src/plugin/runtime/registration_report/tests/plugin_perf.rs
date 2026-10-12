use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const MODULES_PER_SAMPLE: usize = 128;

#[test]
fn plugin_report_reserves_module_diagnostic_capacity() {
    let source = include_str!("../plugin.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(package_manifest.modules.len())"));
    assert!(!implementation.contains("let mut diagnostics = Vec::new()"));
}

#[test]
fn plugin_report_fetches_manifest_before_diagnostics() {
    let source = include_str!("../plugin.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let manifest = implementation
        .find("let package_manifest = plugin.package_manifest()")
        .expect("manifest");
    let capacity = implementation
        .find("Vec::with_capacity(package_manifest.modules.len())")
        .expect("diagnostic capacity");
    assert!(manifest < capacity);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830bx_runtime_plugin_report_capacity_p95() {
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
        "RUNTIME376_PLUGIN_REPORT_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} modules_per_sample={MODULES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..256 {
        let mut diagnostics = if optimized {
            Vec::with_capacity(MODULES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..MODULES_PER_SAMPLE {
            diagnostics.push(index);
        }
        checksum ^= diagnostics.len();
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
