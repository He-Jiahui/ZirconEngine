use std::hint::black_box;
use std::time::Instant;

use super::*;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchInvocation,
    layout::UiPoint,
    surface::{UiPointerEventKind, UiPointerRoute, UiPointerRoutingPath},
    tree::UiDirtyFlags,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826ar_pointer_reply_preserves_phase_and_dirty_effects() {
    let handler = UiNodeId::new(7);
    let mut result = UiPointerDispatchResult::new(empty_route());
    result.handled_by = Some(handler);
    result.invocations = vec![
        invocation(
            handler,
            UiDispatchPhase::Capture,
            UiPointerDispatchEffect::Handled,
        ),
        invocation(
            UiNodeId::new(8),
            UiDispatchPhase::Target,
            UiPointerDispatchEffect::RequestDirty(UiDirtyFlags {
                render: true,
                ..UiDirtyFlags::default()
            }),
        ),
        invocation(
            UiNodeId::new(9),
            UiDispatchPhase::Bubble,
            UiPointerDispatchEffect::RequestDamage(Default::default()),
        ),
    ];

    let reply = pointer_reply(&result, UiPointerId::new(3));

    assert_eq!(reply.phase, Some(UiDispatchPhase::Capture));
    assert_eq!(reply.effects.len(), 1);
    assert!(matches!(
        reply.effects[0],
        UiDispatchEffect::DirtyRedraw { target, dirty, .. }
            if target == UiNodeId::new(8) && dirty.render
    ));

    result.handled_by = None;
    let reply = pointer_reply(&result, UiPointerId::new(3));
    assert_eq!(reply.phase, Some(UiDispatchPhase::Bubble));
}

#[test]
fn optimization_batch_20260826ar_pointer_reply_scans_invocations_once() {
    let source = include_str!("../pointer_reply.rs");
    let reply_path = bounded_source(
        source,
        "pub(super) fn pointer_reply(",
        "pub(super) fn pointer_component_handler",
    );
    let scan_path = bounded_source(
        source,
        "fn scan_pointer_invocations(",
        "fn pointer_release_target",
    );

    assert!(reply_path.contains("scan_pointer_invocations"));
    assert!(!source.contains("fn pointer_reply_phase("));
    assert_eq!(
        scan_path
            .matches("for invocation in &routed_result.invocations")
            .count(),
        1
    );
    assert!(!scan_path.contains(".iter().rev()"));
    assert!(scan_path.contains("handler_phase"));
    assert!(scan_path.contains("redraw_phase"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ar_pointer_reply_single_pass_p95() {
    const INVOCATIONS: usize = 16_384;
    const REPLIES: usize = 128;
    let handler = UiNodeId::new(1);
    let mut result = UiPointerDispatchResult::new(empty_route());
    result.invocations = (1..=INVOCATIONS as u64)
        .map(|node_id| {
            invocation(
                UiNodeId::new(node_id),
                UiDispatchPhase::Bubble,
                UiPointerDispatchEffect::Handled,
            )
        })
        .collect();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(REPLIES, || {
                legacy_scan_checksum(&result, handler)
            }));
            optimized_ns.push(measure_ns(REPLIES, || {
                optimized_scan_checksum(&result, handler)
            }));
        } else {
            optimized_ns.push(measure_ns(REPLIES, || {
                optimized_scan_checksum(&result, handler)
            }));
            legacy_ns.push(measure_ns(REPLIES, || {
                legacy_scan_checksum(&result, handler)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(4),
        "single-pass pointer reply P95 must be at least 20% below the double scan: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME77_POINTER_REPLY_SINGLE_PASS_BENCH_V1 invocations={INVOCATIONS} replies_per_sample={REPLIES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_invocation_visits_per_sample={} optimized_invocation_visits_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        INVOCATIONS * REPLIES * 2,
        INVOCATIONS * REPLIES,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn invocation(
    node_id: UiNodeId,
    phase: UiDispatchPhase,
    effect: UiPointerDispatchEffect,
) -> UiPointerDispatchInvocation {
    UiPointerDispatchInvocation {
        node_id,
        phase,
        effect,
    }
}

fn empty_route() -> UiPointerRoute {
    UiPointerRoute {
        kind: UiPointerEventKind::Move,
        button: None,
        modifiers: Default::default(),
        activation_phase: Default::default(),
        point: UiPoint::new(0.0, 0.0),
        scroll_delta: 0.0,
        target: None,
        hit_path: Default::default(),
        routing_path: UiPointerRoutingPath::from_root_to_leaf(Vec::new()),
        stacked: Vec::new(),
        entered: Vec::new(),
        left: Vec::new(),
        captured: None,
        pressed: None,
        click_target: None,
        release_inside_pressed: false,
        focused: None,
        fallback_to_root: false,
        root_targets: Vec::new(),
    }
}

fn optimized_scan_checksum(result: &UiPointerDispatchResult, handler: UiNodeId) -> usize {
    let mut effects = Vec::new();
    let phases = scan_pointer_invocations(result, Some(handler), &mut effects);
    effects.len()
        + phases.handler_phase.map(phase_checksum).unwrap_or_default()
        + phases.redraw_phase.map(phase_checksum).unwrap_or_default()
}

fn legacy_scan_checksum(result: &UiPointerDispatchResult, handler: UiNodeId) -> usize {
    let dirty_effect_count = result
        .invocations
        .iter()
        .filter(|invocation| {
            matches!(invocation.effect, UiPointerDispatchEffect::RequestDirty(dirty) if dirty.any())
        })
        .count();
    let phase = result
        .invocations
        .iter()
        .rev()
        .find(|invocation| invocation.node_id == handler)
        .map(|invocation| invocation.phase);
    dirty_effect_count + phase.map(phase_checksum).unwrap_or_default()
}

fn phase_checksum(phase: UiDispatchPhase) -> usize {
    match phase {
        UiDispatchPhase::Preprocess => 1,
        UiDispatchPhase::PreviewTunnel => 2,
        UiDispatchPhase::Direct => 3,
        UiDispatchPhase::Target => 4,
        UiDispatchPhase::Bubble => 5,
        UiDispatchPhase::DefaultAction => 6,
        UiDispatchPhase::Capture => 7,
    }
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
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
