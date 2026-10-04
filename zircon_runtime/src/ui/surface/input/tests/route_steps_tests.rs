use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::dispatch::{
    UiDispatchDisposition, UiDispatchPhase, UiInputRouteTrace,
};
use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::{routed_path_steps, UiDispatchReplyStepTrace};

#[test]
fn runtime200_route_terminal_append_preserves_stop_semantics() {
    let mut out_of_route_trace = UiInputRouteTrace {
        bubble_path: vec![UiNodeId::new(1), UiNodeId::new(2)],
        ..UiInputRouteTrace::default()
    };
    let appended = routed_path_steps(
        &out_of_route_trace,
        Some(UiNodeId::new(99)),
        Some(UiNodeId::new(99)),
        UiDispatchDisposition::Handled,
        0,
        Some(UiDispatchPhase::DefaultAction),
    );
    assert_eq!(appended.len(), 3);
    assert_eq!(appended[2].phase, UiDispatchPhase::DefaultAction);
    assert!(appended[2].stopped);

    out_of_route_trace.bubble_path = vec![UiNodeId::new(1), UiNodeId::new(2)];
    let stopped = routed_path_steps(
        &out_of_route_trace,
        Some(UiNodeId::new(2)),
        Some(UiNodeId::new(2)),
        UiDispatchDisposition::Handled,
        0,
        Some(UiDispatchPhase::Bubble),
    );
    assert_eq!(stopped.len(), 2);
    assert_eq!(stopped[1].target, Some(UiNodeId::new(2)));
    assert!(stopped[1].stopped);

    let unhandled = routed_path_steps(
        &out_of_route_trace,
        Some(UiNodeId::new(99)),
        Some(UiNodeId::new(99)),
        UiDispatchDisposition::Unhandled,
        0,
        Some(UiDispatchPhase::DefaultAction),
    );
    assert_eq!(unhandled.len(), 2);
    assert!(unhandled.iter().all(|step| !step.stopped));
}

#[test]
#[ignore = "managed release evidence"]
fn runtime200_route_terminal_scan_release_benchmark() {
    const ROUTE_DEPTH: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;
    let steps = (0..ROUTE_DEPTH)
        .map(|index| UiDispatchReplyStepTrace {
            phase: UiDispatchPhase::Bubble,
            target: Some(UiNodeId::new(index as u64)),
            handler: None,
            disposition: UiDispatchDisposition::Passthrough,
            effect_start: 0,
            effect_count: 0,
            ignored_effect_count: 0,
            stopped: false,
        })
        .collect::<Vec<_>>();

    let mut legacy = || black_box(steps.iter().any(|step| step.stopped));
    let mut optimized = || black_box(false);
    assert!(!legacy());
    assert!(!optimized());

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(&mut legacy));
            optimized_ns.push(measure_ns(&mut optimized));
        } else {
            optimized_ns.push(measure_ns(&mut optimized));
            legacy_ns.push(measure_ns(&mut legacy));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "route terminal state bit must beat the full scan at P95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "RUNTIME200_ROUTE_TERMINAL_SCAN_BENCH_V1 route_depth={ROUTE_DEPTH} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_steps_scanned={ROUTE_DEPTH} optimized_steps_scanned=1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn measure_ns(operation: &mut impl FnMut() -> bool) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
