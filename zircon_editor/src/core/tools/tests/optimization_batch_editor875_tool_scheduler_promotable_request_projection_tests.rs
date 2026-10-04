use std::hint::black_box;
use std::time::Instant;

use super::{
    AcquireOutcome, ReleaseOutcome, ToolInstanceId, ToolLeaseHandle, ToolOwnerGeneration,
    ToolQueueLimits, ToolResourceKey, ToolResourceSet, ToolScheduleReport, ToolScheduler,
};
use zircon_runtime_interface::ui::dispatch::UiWindowId;

const RESOURCE_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor875_promotable_request_projection_preserves_resource_order() {
    let first_resource = ToolResourceKey::modal_surface(UiWindowId::new("editor875.a"));
    let second_resource = ToolResourceKey::modal_surface(UiWindowId::new("editor875.b"));
    let held_resources =
        ToolResourceSet::new([second_resource.clone(), first_resource.clone()]).unwrap();
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(8, 8));
    let holder = acquired(scheduler.acquire(tool("editor875.holder"), held_resources));
    let second_waiter = queued(scheduler.acquire(
        tool("editor875.second"),
        ToolResourceSet::single(second_resource.clone()),
    ));
    let first_waiter = queued(scheduler.acquire(
        tool("editor875.first"),
        ToolResourceSet::single(first_resource.clone()),
    ));

    let mut expected = vec![
        (second_resource, second_waiter.id()),
        (first_resource, first_waiter.id()),
    ];
    expected.sort_by(|left, right| left.0.cmp(&right.0));

    let release = scheduler.release(holder.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = release.outcome()
    else {
        panic!("the held resource set must release");
    };
    assert_eq!(activated_leases.len(), expected.len());
    assert_eq!(
        activated_leases
            .iter()
            .map(ToolLeaseHandle::request_id)
            .collect::<Vec<_>>(),
        expected
            .into_iter()
            .map(|(_, request_id)| request_id)
            .collect::<Vec<_>>()
    );

    let source = include_str!("../scheduler.rs");
    let promotion = source
        .split("fn promote_waiting_singles")
        .nth(1)
        .expect("single promotion owner must exist")
        .split("fn remove_empty_resource_states")
        .next()
        .expect("single promotion owner must stay bounded");
    assert!(promotion.contains("promotable_request_ids"));
    assert!(!promotion.contains("self.resources.keys().cloned().collect"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor875_tool_scheduler_promotable_request_projection_benchmark() {
    let resources = (0..RESOURCE_COUNT)
        .map(|index| format!("editor875.resource.{index:05}.long.identity"))
        .collect::<Vec<_>>();
    let promotable = vec![false; RESOURCE_COUNT];
    assert_eq!(legacy_projection(&resources).len(), RESOURCE_COUNT);
    assert!(optimized_projection(&resources, &promotable).is_empty());

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_projection(&resources)));
            optimized.push(measure(|| optimized_projection(&resources, &promotable)));
        } else {
            optimized.push(measure(|| optimized_projection(&resources, &promotable)));
            legacy.push(measure(|| legacy_projection(&resources)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR875_TOOL_SCHEDULER_PROMOTABLE_REQUEST_PROJECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} resources={RESOURCE_COUNT} promotable_requests=0 legacy_resource_key_clones={RESOURCE_COUNT} optimized_resource_key_clones=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert!(
        optimized_p95 < legacy_p95,
        "expected request-id projection to improve P95, got legacy={legacy_p95}ns optimized={optimized_p95}ns"
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

fn legacy_projection(resources: &[String]) -> Vec<String> {
    resources.to_vec()
}

fn optimized_projection(resources: &[String], promotable: &[bool]) -> Vec<usize> {
    let mut request_ids = Vec::new();
    for (index, (_, is_promotable)) in resources.iter().zip(promotable).enumerate() {
        if !is_promotable {
            continue;
        }
        if request_ids.is_empty() {
            request_ids.reserve(resources.len());
        }
        request_ids.push(index);
    }
    request_ids
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
