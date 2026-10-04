use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const CAPABILITY_ADMISSION_COUNT: usize = 16_384;
const UNIQUE_CAPABILITY_COUNT: usize = 1_024;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn capabilities() -> Vec<String> {
    (0..CAPABILITY_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "runtime.script.capability.{:04}",
                (index * 1_021) % UNIQUE_CAPABILITY_COUNT
            )
        })
        .collect()
}

fn legacy_capability_build(capabilities: &[String]) -> Vec<String> {
    let mut values = Vec::new();
    for capability in capabilities {
        values.push(capability.clone());
        values.sort();
        values.dedup();
    }
    values
}

fn optimized_capability_build(capabilities: &[String]) -> Vec<String> {
    let mut values = Vec::new();
    for capability in capabilities {
        insert_sorted_unique(&mut values, capability.clone());
    }
    values
}

#[test]
fn optimization_batch_20260826u_runtime07_capability_builders_remain_sorted_and_unique() {
    let function = ScriptHostFunctionDescriptor::new("query", 0, 0, ScriptHostValueKind::Null)
        .with_required_capability("zeta")
        .with_required_capability("alpha")
        .with_required_capability("zeta");
    let module = ScriptHostModuleDescriptor::new("weather", "1")
        .with_capability("zeta")
        .with_capability("alpha")
        .with_capability("zeta");

    assert_eq!(function.required_capabilities, vec!["alpha", "zeta"]);
    assert_eq!(module.capabilities, vec!["alpha", "zeta"]);

    let mut externally_populated =
        vec!["zeta".to_string(), "alpha".to_string(), "alpha".to_string()];
    insert_sorted_unique(&mut externally_populated, "beta".to_string());
    assert_eq!(externally_populated, vec!["alpha", "beta", "zeta"]);
}

#[test]
fn optimization_batch_20260826u_runtime07_capability_builders_use_binary_insertion() {
    let source = include_str!("../descriptors.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("fn insert_sorted_unique("));
    assert!(production.contains("values.binary_search(&value)"));
    assert_eq!(production.matches("insert_sorted_unique(").count(), 3);
    assert!(production.contains("values.windows(2).all"));
    assert_eq!(production.matches("values.sort();").count(), 1);
    assert_eq!(production.matches("values.dedup();").count(), 1);
    assert!(!production.contains("required_capabilities.sort()"));
    assert!(!production.contains("required_capabilities.dedup()"));
    assert!(!production.contains("capabilities.sort()"));
    assert!(!production.contains("capabilities.dedup()"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826u_runtime07_capability_binary_insert_performance_evidence() {
    let capabilities = capabilities();
    assert_eq!(
        legacy_capability_build(&capabilities),
        optimized_capability_build(&capabilities)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_capability_build(black_box(&capabilities)));
            legacy_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(optimized_capability_build(black_box(&capabilities)));
            optimized_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(optimized_capability_build(black_box(&capabilities)));
            optimized_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_capability_build(black_box(&capabilities)));
            legacy_samples.push(started.elapsed());
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME07_CAPABILITY_BINARY_INSERT_BENCH_V1 admissions={CAPABILITY_ADMISSION_COUNT} \
             unique_capabilities={UNIQUE_CAPABILITY_COUNT} legacy_full_sort_calls={CAPABILITY_ADMISSION_COUNT} \
             optimized_full_sort_calls=0 legacy_p95_ns={} optimized_p95_ns={}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
        "binary-insert P95 {:?} exceeded 60% of full-sort P95 {:?}",
        optimized_p95,
        legacy_p95,
    );
}
