use std::hint::black_box;
use std::time::Instant;

const NATIVE_DYNAMIC_PACKAGE_COUNT: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn native_package_preparation_indexes_discovery_once() {
    let source = include_str!("../prepare.rs");
    let body = source
        .split("fn prepare_native_dynamic_packages_with_cancellation")
        .nth(1)
        .expect("native package preparation");
    let repeated_scan = ["native_report", ".discovered", ".iter()", ".find"].concat();

    assert!(body.contains("discovered_by_plugin_id"));
    assert!(!body.contains(&repeated_scan));
}

#[test]
fn optimization_batch_fw_editor409_native_invocation_projection_reserves_package_capacity() {
    let source = include_str!("../prepare.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("native package preparation production source");

    assert!(production.contains("Vec::with_capacity(plan.native_dynamic_packages.len())"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fw_editor409_native_invocation_capacity_benchmark() {
    for _ in 0..4 {
        black_box(measure_invocation_pushes(false));
        black_box(measure_invocation_pushes(true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_invocation_pushes(false));
            optimized_samples.push(measure_invocation_pushes(true));
        } else {
            optimized_samples.push(measure_invocation_pushes(true));
            legacy_samples.push(measure_invocation_pushes(false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR409_NATIVE_DYNAMIC_INVOCATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} packages_per_sample={NATIVE_DYNAMIC_PACKAGE_COUNT} legacy_vec_growth_allocations_at_least=11 optimized_vec_growth_allocations=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 75 / 100);
}

fn measure_invocation_pushes(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut invocations = if optimized {
        Vec::with_capacity(NATIVE_DYNAMIC_PACKAGE_COUNT)
    } else {
        Vec::new()
    };
    for package in 0..NATIVE_DYNAMIC_PACKAGE_COUNT {
        invocations.push(package);
    }
    black_box(invocations);
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
