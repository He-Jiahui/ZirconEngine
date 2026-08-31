use std::hint::black_box;
use std::time::Instant;

use super::{
    live_plugin_backend_success_message, ModulePluginLiveHostCommand, ModulePluginLiveHostOutcome,
};

const SAMPLE_PAIRS: usize = 31;
const MESSAGES_PER_SAMPLE: usize = 40_000;

#[test]
fn optimization_batch_20260829x_editor243_plugin_success_message_preserves_bytes() {
    for diagnostics in [Vec::new(), diagnostic_fixture()] {
        let outcome = outcome(diagnostics);
        assert_eq!(
            live_plugin_backend_success_message(&outcome),
            legacy_success_message(&outcome)
        );
    }
}

#[test]
fn optimization_batch_20260829x_editor243_plugin_success_message_uses_one_buffer() {
    let source = include_str!("../dispatch.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("fn live_plugin_backend_success_message")
        .nth(1)
        .expect("success message builder");

    assert!(body.contains("String::with_capacity"));
    assert!(body.contains("message.push_str"));
    assert!(!body.contains("diagnostics.join"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829x_editor243_single_allocation_plugin_success_message_bench() {
    let outcome = outcome(diagnostic_fixture());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &outcome));
            optimized_samples.push(measure(true, &outcome));
        } else {
            optimized_samples.push(measure(true, &outcome));
            legacy_samples.push(measure(false, &outcome));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    let diagnostic_bytes = outcome.diagnostics.iter().map(String::len).sum::<usize>();
    println!(
        "EDITOR243_SINGLE_ALLOCATION_PLUGIN_SUCCESS_MESSAGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
messages_per_sample={MESSAGES_PER_SAMPLE} diagnostic_count={} diagnostic_bytes={diagnostic_bytes} \
legacy_result_allocations_per_message=2 optimized_result_allocations_per_message=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        outcome.diagnostics.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn outcome(diagnostics: Vec<String>) -> ModulePluginLiveHostOutcome {
    ModulePluginLiveHostOutcome {
        plugin_id: "rendering.virtual-geometry.production-pipeline".to_string(),
        command: ModulePluginLiveHostCommand::HotReload,
        diagnostics,
    }
}

fn diagnostic_fixture() -> Vec<String> {
    (0..8)
        .map(|index| {
            format!(
                "module shard {index} reloaded with generation-safe handles and synchronized editor descriptors"
            )
        })
        .collect()
}

fn legacy_success_message(outcome: &ModulePluginLiveHostOutcome) -> String {
    if outcome.diagnostics.is_empty() {
        return format!(
            "Plugin {} {}",
            outcome.plugin_id,
            outcome.command.past_tense()
        );
    }
    format!(
        "Plugin {} {}: {}",
        outcome.plugin_id,
        outcome.command.past_tense(),
        outcome.diagnostics.join("; ")
    )
}

fn measure(optimized: bool, outcome: &ModulePluginLiveHostOutcome) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..MESSAGES_PER_SAMPLE {
        let message = if optimized {
            live_plugin_backend_success_message(black_box(outcome))
        } else {
            legacy_success_message(black_box(outcome))
        };
        checksum = checksum.wrapping_add(black_box(message).len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
