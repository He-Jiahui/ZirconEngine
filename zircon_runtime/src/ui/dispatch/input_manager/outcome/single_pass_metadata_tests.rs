use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchAppliedEffect, UiDispatchEffect, UiDispatchHostRequest,
        UiDispatchHostRequestKind, UiDispatchReply, UiInputDispatchResult, UiInputEvent,
        UiRedrawRequestReason, UiTextInputEvent,
    },
    event_ui::UiNodeId,
    tree::UiDirtyFlags,
};

use super::collect_dispatch_metadata;

const SAMPLE_PAIRS: usize = 21;
const BATCHES_PER_SAMPLE: usize = 2_048;
const RESULTS_PER_BATCH: usize = 256;

#[test]
fn runtime_hotpath_batch_runtime177_182_dispatch_metadata_preserves_requests_and_redraw() {
    let mut first = dispatch_result(1);
    first.host_requests.push(host_request(1));
    let mut second = dispatch_result(2);
    second.host_requests.push(host_request(2));
    second.applied_effects.push(UiDispatchAppliedEffect {
        effect_index: 2,
        effect: UiDispatchEffect::DirtyRedraw {
            target: UiNodeId::new(2),
            dirty: UiDirtyFlags::default(),
            reason: UiRedrawRequestReason::Input,
        },
    });

    let results = vec![first, second];
    let (requests, redraw_requested) = collect_dispatch_metadata(&results, false);
    assert_eq!(
        requests
            .iter()
            .map(|request| request.effect_index)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert!(redraw_requested);

    let (_, inherited_redraw) = collect_dispatch_metadata(&[dispatch_result(3)], true);
    assert!(inherited_redraw);
}

#[test]
fn runtime_hotpath_batch_runtime177_182_dispatch_metadata_uses_single_result_pass() {
    let source = include_str!("../outcome.rs");
    let helper_start = source.find("fn collect_dispatch_metadata").unwrap();
    let helper_end = source[helper_start..]
        .find("impl UiInputDispatchOutcome")
        .map(|offset| helper_start + offset)
        .unwrap();
    let helper_source = &source[helper_start..helper_end];
    assert_eq!(helper_source.matches("for result in results").count(), 1);
    assert!(!helper_source.contains("flat_map"));
    assert!(!helper_source.contains("results.iter().any"));
    assert!(helper_source.contains("host_requests.extend"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime_hotpath_batch_runtime177_182_dispatch_outcome_single_pass_bench() {
    let results = dispatch_fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&results));
            optimized_samples.push(measure_optimized(&results));
        } else {
            optimized_samples.push(measure_optimized(&results));
            legacy_samples.push(measure_legacy(&results));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME177_DISPATCH_OUTCOME_SINGLE_PASS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} results_per_batch={RESULTS_PER_BATCH} \
legacy_result_passes_per_batch=2 optimized_result_passes_per_batch=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single dispatch metadata pass P95 {optimized_p95_ns}ns must be at most 70% of two-pass metadata P95 {legacy_p95_ns}ns"
    );
}

fn dispatch_result(index: usize) -> UiInputDispatchResult {
    UiInputDispatchResult::new(
        UiInputEvent::Text(UiTextInputEvent {
            metadata: Default::default(),
            text: index.to_string(),
        }),
        UiDispatchReply::unhandled(),
    )
}

fn host_request(index: usize) -> UiDispatchHostRequest {
    UiDispatchHostRequest {
        effect_index: index,
        request: UiDispatchHostRequestKind::HighPrecisionPointer {
            target: UiNodeId::new(index as u64),
            enabled: true,
        },
        reason: "performance fixture".to_string(),
    }
}

fn dispatch_fixture() -> Vec<UiInputDispatchResult> {
    let mut results = (0..RESULTS_PER_BATCH)
        .map(dispatch_result)
        .collect::<Vec<_>>();
    results
        .last_mut()
        .unwrap()
        .host_requests
        .push(host_request(RESULTS_PER_BATCH));
    results
}

fn legacy_collect_dispatch_metadata(
    results: &[UiInputDispatchResult],
    initial_redraw_requested: bool,
) -> (Vec<UiDispatchHostRequest>, bool) {
    let host_requests = results
        .iter()
        .flat_map(|result| result.host_requests.iter().cloned())
        .collect();
    let redraw_requested = initial_redraw_requested
        || results.iter().any(|result| {
            result
                .applied_effects
                .iter()
                .any(|applied| matches!(applied.effect, UiDispatchEffect::DirtyRedraw { .. }))
        });
    (host_requests, redraw_requested)
}

fn measure_legacy(results: &[UiInputDispatchResult]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BATCHES_PER_SAMPLE {
        let (requests, redraw) =
            black_box(legacy_collect_dispatch_metadata(black_box(results), false));
        checksum ^= requests.len() + usize::from(redraw);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(results: &[UiInputDispatchResult]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BATCHES_PER_SAMPLE {
        let (requests, redraw) = black_box(collect_dispatch_metadata(black_box(results), false));
        checksum ^= requests.len() + usize::from(redraw);
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
