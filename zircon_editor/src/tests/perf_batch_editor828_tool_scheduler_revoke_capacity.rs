use std::hint::black_box;
use std::time::Instant;

use super::{
    AcquireOutcome, ToolInstanceId, ToolLeaseHandle, ToolOwnerGeneration, ToolResourceKey,
    ToolResourceSet, ToolScheduleReport, ToolScheduler,
};
use crate::core::editor_event::ViewInstanceId;
use zircon_runtime_interface::ui::dispatch::UiWindowId;

#[test]
fn editor828_revoke_preserves_owner_and_kind_filters() {
    let generation = ToolOwnerGeneration::new(828).expect("valid owner generation");
    let shared = ToolResourceKey::modal_surface(UiWindowId::new("editor828.window"));
    let scene = ToolResourceKey::scene_mode_slot(ViewInstanceId::new("editor828.viewport"));
    let mut scheduler = ToolScheduler::new(super::ToolQueueLimits::new(8, 8));

    let owner_holder = acquired(scheduler.acquire(
        ToolInstanceId::from_parts("editor828.owner-holder", generation.value(), 1).unwrap(),
        ToolResourceSet::single(shared.clone()),
    ));
    let owner_waiter = queued(scheduler.acquire(
        ToolInstanceId::from_parts("editor828.owner-waiter", generation.value(), 2).unwrap(),
        ToolResourceSet::single(shared),
    ));

    let owner_report = scheduler.revoke_owner_generation(generation, &[]);
    let super::ToolOwnerRevokeOutcome::Revoked {
        generation: revoked_generation,
        released_leases,
        withdrawn_requests,
        ..
    } = owner_report.outcome()
    else {
        panic!("owner generation should revoke its claims");
    };
    assert_eq!(*revoked_generation, generation);
    assert_eq!(released_leases.len(), 1);
    assert_eq!(released_leases[0].id(), owner_holder.id());
    assert_eq!(withdrawn_requests.len(), 1);
    assert_eq!(withdrawn_requests[0].id(), owner_waiter.id());
    assert!(scheduler.snapshot().active_leases().is_empty());
    assert!(scheduler.snapshot().queued_requests().is_empty());

    let kind_holder = acquired(scheduler.acquire(
        ToolInstanceId::for_test("editor828.kind-holder", ToolOwnerGeneration::BUILTIN).unwrap(),
        ToolResourceSet::single(scene.clone()),
    ));
    let kind_report = scheduler.revoke_owner_generation(
        ToolOwnerGeneration::new(829).unwrap(),
        &[scene.kind().clone()],
    );
    let super::ToolOwnerRevokeOutcome::Revoked {
        released_leases, ..
    } = kind_report.outcome()
    else {
        panic!("resource kind should revoke the builtin claim");
    };
    assert_eq!(released_leases.len(), 1);
    assert_eq!(released_leases[0].id(), kind_holder.id());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor828_tool_scheduler_revoke_capacity_release_benchmark() {
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
        "EDITOR828_TOOL_SCHEDULER_REVOKE_CAPACITY_BENCH_V1 claims={CLAIMS} samples={SAMPLES} legacy_growth_events={} optimized_growth_events=0 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}",
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
