use std::hint::black_box;
use std::time::Instant;

use super::{
    AcquireOutcome, ToolInstanceId, ToolLeaseHandle, ToolLifecycleEvent, ToolOwnerGeneration,
    ToolQueueLimits, ToolResourceKey, ToolResourceSet, ToolScheduleReport, ToolScheduler,
};
use zircon_runtime_interface::ui::dispatch::UiWindowId;

const REQUEST_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor876_shutdown_owned_drain_preserves_request_order_and_positions() {
    let resource = ToolResourceSet::single(ToolResourceKey::modal_surface(UiWindowId::new(
        "editor876.window",
    )));
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(8, 8));
    acquired(scheduler.acquire(tool("editor876.holder"), resource.clone()));
    let first = queued(scheduler.acquire(tool("editor876.first"), resource.clone()));
    let second = queued(scheduler.acquire(tool("editor876.second"), resource));

    let report = scheduler.shutdown();
    let withdrawn = report
        .events()
        .iter()
        .filter_map(|event| match event {
            ToolLifecycleEvent::Withdrawn {
                request,
                previous_position,
            } => Some((request.id(), *previous_position)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(withdrawn, vec![(first.id(), 1), (second.id(), 2)]);
    assert!(scheduler.snapshot().active_leases().is_empty());
    assert!(scheduler.snapshot().queued_requests().is_empty());

    let source = include_str!("../scheduler.rs");
    let shutdown = source
        .split("pub(crate) fn shutdown")
        .nth(1)
        .expect("shutdown owner must exist")
        .split("fn report_existing_claim")
        .next()
        .expect("shutdown owner must stay bounded");
    assert!(shutdown.contains("std::mem::take(&mut self.requests)"));
    assert!(!shutdown.contains(".values()\n            .cloned()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor876_tool_scheduler_shutdown_owned_drain_benchmark() {
    let requests = (0..REQUEST_COUNT)
        .map(|index| format!("editor876.request.{index:05}.long.identity"))
        .collect::<Vec<_>>();
    let positions = (1..=REQUEST_COUNT).collect::<Vec<_>>();
    assert_eq!(legacy_projection(&requests).len(), REQUEST_COUNT);
    assert_eq!(optimized_projection(&positions), REQUEST_COUNT);

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_projection(&requests)));
            optimized.push(measure(|| optimized_projection(&positions)));
        } else {
            optimized.push(measure(|| optimized_projection(&positions)));
            legacy.push(measure(|| legacy_projection(&requests)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR876_TOOL_SCHEDULER_SHUTDOWN_OWNED_DRAIN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} queued_requests={REQUEST_COUNT} legacy_request_handle_clones={REQUEST_COUNT} optimized_request_handle_clones=0 legacy_handle_scratch_slots={REQUEST_COUNT} optimized_handle_scratch_slots=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95 < legacy_p95,
        "expected owned drain model to improve P95, got legacy={legacy_p95}ns optimized={optimized_p95}ns"
    );
}

fn acquired(report: ToolScheduleReport<AcquireOutcome>) -> ToolLeaseHandle {
    match report.into_parts().0 {
        AcquireOutcome::Acquired { lease } => lease,
        outcome => panic!("expected acquired lease, got {outcome:?}"),
    }
}

fn queued(report: ToolScheduleReport<AcquireOutcome>) -> super::ToolRequestHandle {
    match report.into_parts().0 {
        AcquireOutcome::Queued { request, .. } => request,
        outcome => panic!("expected queued request, got {outcome:?}"),
    }
}

fn tool(definition: &str) -> ToolInstanceId {
    ToolInstanceId::for_test(definition, ToolOwnerGeneration::BUILTIN).unwrap()
}

fn legacy_projection(requests: &[String]) -> Vec<(String, usize)> {
    requests
        .iter()
        .enumerate()
        .map(|(index, request)| (request.clone(), index + 1))
        .collect()
}

fn optimized_projection(positions: &[usize]) -> usize {
    positions.iter().copied().map(black_box).count()
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
