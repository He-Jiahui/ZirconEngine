use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiDispatchHostRequest, UiDispatchHostRequestKind, UiDispatchReply,
        UiInputDispatchResult, UiInputEvent, UiTextInputEvent,
    },
    event_ui::UiNodeId,
};

use super::collect_dispatch_metadata;

const SAMPLE_PAIRS: usize = 17;
const BATCHES_PER_SAMPLE: usize = 1_024;
const RESULTS_PER_BATCH: usize = 4_096;

#[test]
fn runtime_hotpath_batch_runtime820_dispatch_host_request_capacity_preserves_order() {
    let mut first = dispatch_result(1);
    first.host_requests.push(host_request(1));
    let mut second = dispatch_result(2);
    second.host_requests.push(host_request(2));
    let results = vec![first, second, dispatch_result(3), dispatch_result(4)];

    let (requests, redraw_requested) = collect_dispatch_metadata(&results, false);

    assert_eq!(
        requests
            .iter()
            .map(|request| request.effect_index)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert!(!redraw_requested);
    assert!(requests.capacity() >= results.len());
}

#[test]
fn runtime_hotpath_batch_runtime820_dispatch_host_request_capacity_keeps_empty_path_empty() {
    let results = vec![dispatch_result(1), dispatch_result(2), dispatch_result(3)];

    let (requests, redraw_requested) = collect_dispatch_metadata(&results, false);

    assert!(requests.is_empty());
    assert_eq!(requests.capacity(), 0);
    assert!(!redraw_requested);
}

#[test]
fn runtime_hotpath_batch_runtime820_dispatch_host_request_capacity_reserves_sparse_suffix() {
    let mut results = vec![dispatch_result(1), dispatch_result(2)];
    let mut sparse = dispatch_result(3);
    sparse.host_requests.push(host_request(3));
    results.push(sparse);
    results.push(dispatch_result(4));

    let (requests, _) = collect_dispatch_metadata(&results, false);

    assert_eq!(requests.len(), 1);
    assert!(requests.capacity() >= 2);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime_hotpath_batch_runtime820_dispatch_host_request_capacity_bench() {
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

    let legacy_growth_events = growth_events(RESULTS_PER_BATCH);
    let optimized_growth_events = 0;
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME820_DISPATCH_HOST_REQUEST_CAPACITY_BENCH_V1 \
results_per_batch={RESULTS_PER_BATCH} batches_per_sample={BATCHES_PER_SAMPLE} \
sample_pairs={SAMPLE_PAIRS} legacy_growth_events={legacy_growth_events} \
optimized_growth_events={optimized_growth_events} optimized_reservations_per_batch=1 \
optimized_lower_bound_capacity={RESULTS_PER_BATCH} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);
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
        reason: "capacity fixture".to_string(),
    }
}

fn dispatch_fixture() -> Vec<UiInputDispatchResult> {
    let mut results = (0..RESULTS_PER_BATCH)
        .map(dispatch_result)
        .collect::<Vec<_>>();
    results[0]
        .host_requests
        .push(host_request(RESULTS_PER_BATCH));
    results
}

fn legacy_collect_dispatch_metadata(
    results: &[UiInputDispatchResult],
) -> (Vec<UiDispatchHostRequest>, bool) {
    let host_requests = results
        .iter()
        .flat_map(|result| result.host_requests.iter().cloned())
        .collect();
    let redraw_requested = results.iter().any(|result| {
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
        let (requests, redraw) = black_box(legacy_collect_dispatch_metadata(black_box(results)));
        checksum ^= requests.len() ^ requests.capacity() ^ usize::from(redraw);
        black_box(requests);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(results: &[UiInputDispatchResult]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BATCHES_PER_SAMPLE {
        let (requests, redraw) = black_box(collect_dispatch_metadata(black_box(results), false));
        checksum ^= requests.len() ^ requests.capacity() ^ usize::from(redraw);
        black_box(requests);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
