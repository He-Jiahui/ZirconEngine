use std::hint::black_box;
use std::time::Instant;

use super::{
    AcquireOutcome, ReleaseOutcome, ToolInstanceId, ToolLeaseHandle, ToolOwnerGeneration,
    ToolQueueLimits, ToolResourceKey, ToolResourceSet, ToolScheduleReport, ToolScheduler,
};
use crate::core::editor_event::ViewInstanceId;
use zircon_runtime_interface::ui::dispatch::UiWindowId;

#[test]
fn editor827_release_promotes_set_and_single_waiters_in_order() {
    let shared = ToolResourceKey::modal_surface(UiWindowId::new("editor827.window"));
    let scene = ToolResourceKey::scene_mode_slot(ViewInstanceId::new("editor827.viewport"));
    let held_resources = ToolResourceSet::new([shared.clone(), scene.clone()]).unwrap();
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(8, 8));

    let holder = acquired(scheduler.acquire(tool("editor827.holder"), held_resources.clone()));
    let set_waiter = queued(scheduler.acquire(tool("editor827.set-waiter"), held_resources));
    let single_waiter = queued(scheduler.acquire(
        tool("editor827.single-waiter"),
        ToolResourceSet::single(shared),
    ));

    let first_release = scheduler.release(holder.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = first_release.outcome()
    else {
        panic!("the held set must release");
    };
    assert_eq!(activated_leases.len(), 1);
    assert_eq!(activated_leases[0].request_id(), set_waiter.id());

    let second_release = scheduler.release(activated_leases[0].id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = second_release.outcome()
    else {
        panic!("the promoted set must release");
    };
    assert_eq!(activated_leases.len(), 1);
    assert_eq!(activated_leases[0].request_id(), single_waiter.id());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor827_tool_scheduler_promotion_capacity_release_benchmark() {
    const CLAIMS: usize = 4_096;
    const SAMPLES: usize = 17;
    let mut legacy = Vec::with_capacity(SAMPLES);
    let mut optimized = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        if sample % 2 == 0 {
            legacy.push(measure_growth(CLAIMS, false));
            optimized.push(measure_growth(CLAIMS, true));
        } else {
            optimized.push(measure_growth(CLAIMS, true));
            legacy.push(measure_growth(CLAIMS, false));
        }
    }
    let legacy_p95 = percentile(&legacy, 95);
    let optimized_p95 = percentile(&optimized, 95);
    assert!(optimized_p95 <= legacy_p95);
    println!(
        "EDITOR827_TOOL_SCHEDULER_PROMOTION_BENCH_V1 claims={CLAIMS} samples={SAMPLES} legacy_growth_events={} optimized_growth_events=0 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}",
        growth_events(CLAIMS),
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

fn measure_growth(length: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..64 {
        let mut values = if optimized {
            Vec::with_capacity(length)
        } else {
            Vec::new()
        };
        for value in 0..length {
            values.push(black_box(value));
        }
        checksum = checksum.wrapping_add(values.len());
        black_box(values);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(length: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for index in 1..=length {
        if index > capacity {
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
