use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::picking::HitRecord;
use crate::core::framework::render::RenderViewportHandle;

use super::*;

fn location(pointer: PointerId) -> PointerLocation {
    PointerLocation::new(pointer, RenderViewportHandle::new(7), Vec2::new(16.0, 24.0))
}

fn hit(target: HitTarget) -> HitRecord {
    HitRecord::new(target, HitData::new(3, 0.25, None, None))
}

fn dragging_state(targets: &[HitTarget]) -> PointerButtonEventState {
    PointerButtonEventState {
        pressing: BTreeMap::new(),
        dragging: targets
            .iter()
            .copied()
            .map(|target| {
                (
                    target,
                    DragState {
                        start_position: Vec2::new(1.0, 2.0),
                        latest_position: Vec2::new(3.0, 5.0),
                    },
                )
            })
            .collect(),
        dragging_over: BTreeMap::new(),
    }
}

#[test]
fn optimization_batch_hq_runtime598_streamed_drag_target_dispatch_preserves_order() {
    let pointer = PointerId::new(11);
    let dragged = [HitTarget::renderable(10), HitTarget::renderable(20)];
    let drop_target = HitTarget::scene_gizmo(30);
    let pointer_location = location(pointer);
    let locations = HashMap::from([(pointer, pointer_location)]);

    let mut enter_state = PickingEventState::default();
    enter_state
        .button_states
        .insert((pointer, PointerButton::Primary), dragging_state(&dragged));
    let current_hover = PickingHoverMap::new(pointer, vec![hit(drop_target)]);
    let mut enter_events = Vec::new();
    enter_state.dispatch_current_hovers(
        &PickingHoverMap::default(),
        &current_hover,
        &locations,
        &mut enter_events,
    );
    let entered = enter_events
        .iter()
        .filter_map(|event| match &event.kind {
            PickingEventKind::DragEnter { dragged, .. } => Some(*dragged),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(entered, dragged);

    let mut leave_state = PickingEventState::default();
    let mut button_state = dragging_state(&dragged);
    button_state
        .dragging_over
        .insert(drop_target, hit(drop_target).hit);
    leave_state
        .button_states
        .insert((pointer, PointerButton::Primary), button_state);
    let previous_hover = PickingHoverMap::new(pointer, vec![hit(drop_target)]);
    let mut leave_events = Vec::new();
    leave_state.dispatch_exits(
        &previous_hover,
        &PickingHoverMap::default(),
        &locations,
        &mut leave_events,
    );
    let left = leave_events
        .iter()
        .filter_map(|event| match &event.kind {
            PickingEventKind::DragLeave { dragged, .. } => Some(*dragged),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(left, dragged);
}

#[test]
fn optimization_batch_hq_runtime598_drag_hover_dispatch_has_no_target_snapshot() {
    let source = include_str!("../../pointer_event_state.rs");
    let exits = source
        .split("fn dispatch_exits")
        .nth(1)
        .expect("dispatch exits")
        .split("fn dispatch_current_hovers")
        .next()
        .expect("bounded dispatch exits");
    let hovers = source
        .split("fn dispatch_current_hovers")
        .nth(1)
        .expect("dispatch current hovers")
        .split("fn dispatch_input")
        .next()
        .expect("bounded dispatch current hovers");

    for body in [exits, hovers] {
        assert!(body.contains("for dragged in state.dragging.keys().copied()"));
        assert!(!body.contains("collect::<Vec<_>>()"));
    }
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hq_runtime598_drag_target_stream_performance_evidence() {
    fn legacy_emit(targets: &BTreeMap<u64, u64>) -> u64 {
        let snapshot = targets.keys().copied().collect::<Vec<_>>();
        black_box(&snapshot);
        snapshot
            .into_iter()
            .fold(0_u64, |checksum, target| checksum ^ target.rotate_left(7))
    }

    fn streamed_emit(targets: &BTreeMap<u64, u64>) -> u64 {
        targets
            .keys()
            .copied()
            .fold(0_u64, |checksum, target| checksum ^ target.rotate_left(7))
    }

    let targets = (0..32_768_u64)
        .map(|target| (target, target.wrapping_mul(17)))
        .collect::<BTreeMap<_, _>>();
    const SAMPLE_PAIRS: usize = 17;
    let measure_legacy = || {
        let started = Instant::now();
        black_box(legacy_emit(black_box(&targets)));
        started.elapsed().as_nanos().max(1)
    };
    let measure_streamed = || {
        let started = Instant::now();
        black_box(streamed_emit(black_box(&targets)));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_streamed());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut streamed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            streamed_samples.push(measure_streamed());
        } else {
            streamed_samples.push(measure_streamed());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    streamed_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let streamed_p50 = streamed_samples[8];
    let streamed_p95 = streamed_samples[16];
    println!(
        "RUNTIME598_DRAG_TARGET_STREAM_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 streamed_first_pairs=8 targets={} legacy_p50_ns={} legacy_p95_ns={} streamed_p50_ns={} streamed_p95_ns={} legacy_temporary_targets={} streamed_temporary_targets=0 target_ratio_bp=9000",
        targets.len(),
        legacy_p50,
        legacy_p95,
        streamed_p50,
        streamed_p95,
        targets.len(),
    );
    assert!(
        streamed_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(9_000),
        "streamed drag target P95 {streamed_p95} ns exceeded 90% of legacy {legacy_p95} ns"
    );
}
