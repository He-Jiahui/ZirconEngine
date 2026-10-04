use super::{acquired, modal_surface, queued, resources, scene_mode_slot, tool, viewport_input};
use crate::core::tools::{ToolLeaseHandle, ToolQueueLimits, ToolResourceSet, ToolScheduler};
use std::hint::black_box;
use std::time::Instant;

#[test]
fn optimization_batch_r6_wave7_editor643_snapshot_queues_reserve_known_depth() {
    let source = include_str!("../scheduler.rs");
    let snapshot_start = source
        .find("pub fn snapshot(&self)")
        .expect("snapshot function");
    let snapshot_end = source[snapshot_start..]
        .find("\n    pub fn active_input_capture(")
        .map(|offset| snapshot_start + offset)
        .expect("snapshot function boundary");
    let snapshot = &source[snapshot_start..snapshot_end];

    assert!(snapshot.contains("let mut queued = Vec::with_capacity(state.queue.len());"));
    assert!(snapshot.contains(".filter_map(|request_id| self.requests.get(request_id).cloned()),"));
    assert!(snapshot.contains("queued.into_boxed_slice()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_r6_wave7_editor643_snapshot_queue_capacity_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const QUEUED_REQUESTS: usize = 65_536;
    let queue = (0..QUEUED_REQUESTS as u64).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(editor643_measure_queue_projection(&queue, false));
            optimized_samples.push(editor643_measure_queue_projection(&queue, true));
        } else {
            optimized_samples.push(editor643_measure_queue_projection(&queue, true));
            legacy_samples.push(editor643_measure_queue_projection(&queue, false));
        }
    }

    let legacy_p95 = editor643_p95(&legacy_samples);
    let optimized_p95 = editor643_p95(&optimized_samples);
    println!(
        "EDITOR643_PREALLOCATED_TOOL_SNAPSHOT_QUEUE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} queued_requests={QUEUED_REQUESTS} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
        "preallocated snapshot queues must be at least 15% faster at P95"
    );
}

fn editor643_measure_queue_projection(queue: &[u64], preallocated: bool) -> u128 {
    let started = Instant::now();
    let projected = if preallocated {
        let mut projected = Vec::with_capacity(queue.len());
        projected.extend(queue.iter().filter_map(|request| black_box(Some(*request))));
        projected
    } else {
        queue
            .iter()
            .filter_map(|request| black_box(Some(*request)))
            .collect::<Vec<_>>()
    };
    let elapsed = started.elapsed().as_nanos().max(1);
    black_box(projected);
    elapsed
}

fn editor643_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100) - 1]
}

#[test]
fn snapshot_captures_resource_views_and_canonical_claim_handles() {
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(4, 4));
    let viewport_resource = viewport_input("viewport.main");
    let viewport_holder = acquired(scheduler.acquire(
        tool("tool.viewport-holder"),
        ToolResourceSet::single(viewport_resource.clone()),
    ));
    let viewport_waiter = queued(scheduler.acquire(
        tool("tool.viewport-waiter"),
        ToolResourceSet::single(viewport_resource.clone()),
    ));
    let active_set = acquired(scheduler.acquire(
        tool("tool.active-set"),
        resources([
            modal_surface("window.main"),
            scene_mode_slot("viewport.main"),
        ]),
    ));

    let snapshot = scheduler.snapshot();

    assert_eq!(snapshot.resources().len(), 3);
    assert_eq!(
        snapshot.active_leases(),
        [viewport_holder.clone(), active_set]
    );
    assert_eq!(snapshot.queued_requests(), [viewport_waiter.clone()]);
    assert_eq!(
        snapshot
            .resource(&viewport_resource)
            .and_then(|state| state.holder())
            .map(ToolLeaseHandle::id),
        Some(viewport_holder.id())
    );
    assert_eq!(
        snapshot.resource(&viewport_resource).unwrap().queued(),
        [viewport_waiter]
    );
}

#[test]
fn snapshots_are_immutable_receipts_of_capture_time() {
    let viewport_resource = viewport_input("viewport.main");
    let resources = ToolResourceSet::single(viewport_resource.clone());
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(2, 2));
    let holder = acquired(scheduler.acquire(tool("tool.holder"), resources.clone()));
    let next = queued(scheduler.acquire(tool("tool.next"), resources));
    let before_release = scheduler.snapshot();

    scheduler.release(holder.id());
    let after_release = scheduler.snapshot();

    assert_eq!(
        before_release
            .resource(&viewport_resource)
            .and_then(|state| state.holder())
            .map(ToolLeaseHandle::id),
        Some(holder.id())
    );
    assert_eq!(
        before_release
            .resource(&viewport_resource)
            .unwrap()
            .queued(),
        [next.clone()]
    );
    assert_eq!(
        after_release
            .resource(&viewport_resource)
            .and_then(|state| state.holder())
            .map(ToolLeaseHandle::request_id),
        Some(next.id())
    );
}
