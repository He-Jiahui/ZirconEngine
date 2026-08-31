use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::dispatch::UiInputRoutePolicy;

use super::{UI_INPUT_ROUTE_ORDER, route_stage_names_for_policy};

const PERF_MARKER: &str = "RUNTIME366_ROUTE_STAGE_NAMES_CAPACITY_BENCH_V1";

#[test]
fn optimization_batch_20260830bn_runtime_route_stage_names_preserves_policy_order() {
    assert_eq!(
        route_stage_names_for_policy(UiInputRoutePolicy::Bubble),
        vec![
            "popup_stack",
            "preview_tunnel",
            "direct_target",
            "bubble_path"
        ]
    );
    assert_eq!(
        route_stage_names_for_policy(UiInputRoutePolicy::PointerCapture),
        vec!["pointer_capture"]
    );
}

#[test]
fn optimization_batch_20260830bn_runtime_route_stage_names_source_contract() {
    let source = include_str!("../routing.rs");
    assert!(source.contains("let mut names = Vec::with_capacity(4)"));
    assert!(source.contains("for stage in UI_INPUT_ROUTE_ORDER"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bn_runtime_route_stage_names_p95() {
    const MATCHES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..MATCHES {
                let mut names = if pass == 0 {
                    Vec::new()
                } else {
                    Vec::with_capacity(4)
                };
                for stage in &UI_INPUT_ROUTE_ORDER[1..=4] {
                    names.push(*stage as usize);
                }
                checksum += names.len() + names.capacity();
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
