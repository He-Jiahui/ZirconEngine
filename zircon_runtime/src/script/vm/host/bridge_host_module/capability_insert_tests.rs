use std::hint::black_box;
use std::time::Instant;

use super::insert_required_capability;

const MARKER: &str = "RUNTIME244_BRIDGE_CAPABILITY_BINARY_INSERT_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const CAPABILITY_COUNT: usize = 512;
const REPEATS: usize = 64;

#[test]
fn optimization_batch_20260826gx_runtime244_bridge_capabilities_stay_unique_and_sorted() {
    let mut capabilities = vec!["bridge.call".to_string()];
    for capability in ["zeta", "alpha", "bridge.call", "alpha", "middle"] {
        insert_required_capability(&mut capabilities, capability.to_string());
    }

    assert_eq!(
        capabilities.iter().map(String::as_str).collect::<Vec<_>>(),
        ["alpha", "bridge.call", "middle", "zeta"]
    );
}

#[test]
fn optimization_batch_20260826gx_runtime244_bridge_capabilities_use_binary_insertion() {
    let source = include_str!("../bridge_host_module.rs");
    let implementation = source
        .split("fn insert_required_capability")
        .nth(1)
        .and_then(|tail| tail.split("pub fn register_bridge_host_module").next())
        .expect("bridge capability insertion implementation");
    assert!(implementation.contains("binary_search(&capability)"));
    assert!(implementation.contains("required_capabilities.insert(index, capability)"));
    assert!(!implementation.contains(".sort"));
    assert!(!implementation.contains(".dedup"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gx_runtime244_bridge_capability_binary_insert_bench() {
    let capabilities = (0..CAPABILITY_COUNT)
        .map(|index| format!("bridge.capability.{index:04}"))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&capabilities, legacy_insert_required_capability));
            optimized_samples.push(measure(&capabilities, insert_required_capability));
        } else {
            optimized_samples.push(measure(&capabilities, insert_required_capability));
            legacy_samples.push(measure(&capabilities, legacy_insert_required_capability));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "binary insertion must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_insert_required_capability(required_capabilities: &mut Vec<String>, capability: String) {
    required_capabilities.push(capability);
    required_capabilities.sort();
    required_capabilities.dedup();
}

fn measure(capabilities: &[String], implementation: fn(&mut Vec<String>, String)) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        let mut required_capabilities = Vec::with_capacity(capabilities.len());
        for capability in black_box(capabilities) {
            implementation(&mut required_capabilities, capability.clone());
        }
        checksum = checksum.wrapping_add(required_capabilities.len());
        black_box(&required_capabilities);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
