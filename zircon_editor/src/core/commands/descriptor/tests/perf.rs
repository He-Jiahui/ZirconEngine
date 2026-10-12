use std::hint::black_box;
use std::time::Instant;

use super::*;

fn descriptor() -> EditorCommandDescriptor {
    EditorCommandDescriptor::operation(
        EditorOperationPath::parse("editor.performance.capabilities").unwrap(),
    )
}

fn capability_fixture(offset: usize) -> Vec<String> {
    (0..4_096)
        .map(|index| {
            let permuted = (index * 3_571 + offset) % 6_151;
            format!(
                "editor.performance.capability.{permuted:05}.{}",
                "retained-capability-name".repeat(2)
            )
        })
        .collect()
}

fn legacy_descriptor(first: &[String], second: &[String]) -> EditorCommandDescriptor {
    let mut descriptor = descriptor();
    descriptor
        .required_capabilities
        .extend(first.iter().cloned());
    descriptor.required_capabilities.sort();
    descriptor.required_capabilities.dedup();
    descriptor
        .required_capabilities
        .extend(second.iter().cloned());
    descriptor.required_capabilities.sort();
    descriptor.required_capabilities.dedup();
    descriptor
}

#[test]
fn optimization_batch_hu_editor603_chained_capabilities_remain_sorted_and_unique() {
    let descriptor = descriptor()
        .with_required_capabilities(["editor.zeta", "editor.alpha", "editor.zeta"])
        .with_required_capabilities(["editor.beta", "editor.alpha"]);

    assert_eq!(
        descriptor.required_capabilities(),
        &["editor.alpha", "editor.beta", "editor.zeta"]
    );
}

#[test]
fn optimization_batch_hu_editor603_capability_normalization_reserves_and_uses_unstable_sort() {
    let source = include_str!("../../descriptor.rs");
    let body = source
        .split("pub fn with_required_capabilities")
        .nth(1)
        .expect("required capability builder")
        .split("pub fn with_execution_contract")
        .next()
        .expect("bounded required capability builder");

    assert!(body.contains("size_hint()"));
    assert!(body.contains("reserve(lower_bound)"));
    assert!(body.contains("sort_unstable()"));
    assert!(!body.contains("required_capabilities.sort();"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hu_editor603_capability_normalization_performance_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    let first = capability_fixture(0);
    let second = capability_fixture(2_047);
    let measure_legacy = || {
        let started = Instant::now();
        black_box(legacy_descriptor(&first, &second));
        started.elapsed().as_nanos().max(1)
    };
    let measure_unstable = || {
        let started = Instant::now();
        black_box(
            descriptor()
                .with_required_capabilities(first.iter().cloned())
                .with_required_capabilities(second.iter().cloned()),
        );
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_unstable());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut unstable_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            unstable_samples.push(measure_unstable());
        } else {
            unstable_samples.push(measure_unstable());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    unstable_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let unstable_p50 = unstable_samples[8];
    let unstable_p95 = unstable_samples[16];
    println!(
        "EDITOR603_CAPABILITY_NORMALIZATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 unstable_first_pairs=8 capabilities_per_call={} chained_calls=2 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} unstable_p50_ns={unstable_p50} unstable_p95_ns={unstable_p95} stable_sort=1->0 target_ratio_bp=9000",
        first.len(),
    );
    assert!(
        unstable_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(9_000),
        "unstable capability normalization P95 {unstable_p95} ns exceeded 90% of legacy {legacy_p95} ns"
    );
}
