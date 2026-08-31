use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchReply, UiInputEvent, UiInputEventMetadata, UiToastTimerInputEvent},
    event_ui::UiNodeId,
};

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828id_runtime_tooltip_dismiss_moves_event_and_notes() {
    let result = benchmark_result(4 * 1024);
    let event_allocation = toast_id(&result.event).as_ptr();
    let note_allocation = result.diagnostics.notes[0].as_ptr();

    let (event, notes) = into_tooltip_dismiss_parts(result);

    assert_eq!(toast_id(&event).as_ptr(), event_allocation);
    assert_eq!(notes[0].as_ptr(), note_allocation);
}

#[test]
fn optimization_batch_20260828id_runtime_tooltip_dismiss_consumes_owned_result() {
    let source = include_str!("../tooltip.rs");
    let dispatch = source
        .split("pub(super) fn dispatch_tooltip_dismiss")
        .nth(1)
        .and_then(|body| body.split("fn into_tooltip_dismiss_parts").next())
        .expect("tooltip dismiss dispatch implementation");
    let conversion = source
        .split("fn into_tooltip_dismiss_parts")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("owned result conversion");

    assert!(dispatch.contains("into_tooltip_dismiss_parts(result)"));
    assert!(!dispatch.contains("result.event.clone()"));
    assert!(conversion.contains("UiInputDispatchResult {"));
    assert!(conversion.contains("event, diagnostics, .."));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828id_runtime_owned_tooltip_result_event_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_tooltip_dismiss_parts(benchmark_result(64 * 1024)));
    black_box(into_tooltip_dismiss_parts(benchmark_result(64 * 1024)));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS)
            .map(|_| benchmark_result(64 * 1024))
            .collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS)
            .map(|_| benchmark_result(64 * 1024))
            .collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_results(legacy_inputs, legacy_tooltip_dismiss_parts));
            optimized_samples.push(measure_results(
                optimized_inputs,
                into_tooltip_dismiss_parts,
            ));
        } else {
            optimized_samples.push(measure_results(
                optimized_inputs,
                into_tooltip_dismiss_parts,
            ));
            legacy_samples.push(measure_results(legacy_inputs, legacy_tooltip_dismiss_parts));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME276_OWNED_TOOLTIP_DISMISS_RESULT_EVENT_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_result(id_bytes: usize) -> UiInputDispatchResult {
    let event = UiInputEvent::ToastTimer(UiToastTimerInputEvent {
        metadata: UiInputEventMetadata::default(),
        target: UiNodeId::new(83),
        toast_id: "tooltip-dismiss-event/".repeat(id_bytes / 22),
    });
    let mut result = UiInputDispatchResult::new(event, UiDispatchReply::unhandled());
    result
        .diagnostics
        .notes
        .push("tooltip-dismiss-note/".repeat(id_bytes / 21));
    result
}

fn toast_id(event: &UiInputEvent) -> &str {
    let UiInputEvent::ToastTimer(event) = event else {
        panic!("expected toast timer fixture");
    };
    event.toast_id.as_str()
}

fn legacy_tooltip_dismiss_parts(result: UiInputDispatchResult) -> (UiInputEvent, Vec<String>) {
    let event = result.event.clone();
    let notes = result.diagnostics.notes;
    (event, notes)
}

fn measure_results(
    results: Vec<UiInputDispatchResult>,
    mut convert: impl FnMut(UiInputDispatchResult) -> (UiInputEvent, Vec<String>),
) -> u128 {
    let started = Instant::now();
    for result in results {
        black_box(convert(black_box(result)));
    }
    started.elapsed().as_nanos()
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
