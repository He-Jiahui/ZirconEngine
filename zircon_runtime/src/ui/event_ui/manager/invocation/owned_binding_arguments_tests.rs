use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::binding::{
    UiBindingCall, UiBindingValue, UiEventBinding, UiEventKind, UiEventPath,
};

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hu_runtime_moves_owned_binding_argument_allocation() {
    let mut binding = benchmark_binding(8, 64);
    let expected = binding.action.as_ref().unwrap().arguments.clone();
    let allocation = binding.action.as_ref().unwrap().arguments.as_ptr();

    let arguments = take_binding_arguments(&mut binding);

    assert_eq!(arguments, expected);
    assert_eq!(arguments.as_ptr(), allocation);
    assert!(binding.action.as_ref().unwrap().arguments.is_empty());
}

#[test]
fn optimization_batch_20260828hu_runtime_binding_invocation_uses_owned_arguments() {
    let source = include_str!("../invocation.rs");
    let invoke_binding = source
        .split("pub fn invoke_binding")
        .nth(1)
        .and_then(|body| body.split("pub fn call_action").next())
        .expect("binding invocation implementation");

    assert!(invoke_binding.contains("mut binding: UiEventBinding"));
    assert!(invoke_binding.contains("Some(binding),"));
    assert!(invoke_binding.contains("take_binding_arguments(&mut binding)"));
    assert!(!invoke_binding.contains("call.arguments.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hu_runtime_owned_binding_arguments_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let binding = benchmark_binding(512, 4 * 1024);

    black_box(legacy_clone_binding_arguments(&binding));
    let mut warmup = binding.clone();
    black_box(take_binding_arguments(&mut warmup));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let mut optimized_inputs = (0..ITERATIONS).map(|_| binding.clone()).collect::<Vec<_>>();
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_clone_binding_arguments(black_box(&binding)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for input in &mut optimized_inputs {
                black_box(take_binding_arguments(black_box(input)));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME267_OWNED_UI_BINDING_ARGUMENTS_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_binding(argument_count: usize, argument_bytes: usize) -> UiEventBinding {
    let suffix = "x".repeat(argument_bytes);
    let mut call = UiBindingCall::new("ApplyLargeBinding");
    call.arguments = (0..argument_count)
        .map(|index| UiBindingValue::string(format!("argument-{index}-{suffix}")))
        .collect();
    UiEventBinding::new(
        UiEventPath::new("BenchmarkView", "ApplyButton", UiEventKind::Click),
        call,
    )
}

fn legacy_clone_binding_arguments(binding: &UiEventBinding) -> Vec<UiBindingValue> {
    binding
        .action
        .as_ref()
        .map(|call| call.arguments.clone())
        .unwrap_or_default()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
