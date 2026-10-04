use std::{hint::black_box, time::Instant};

use zircon_runtime_interface::ui::{dispatch::UiInputTimestamp, event_ui::UiNodeId};

use super::UiInputTimerState;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826al_input_timer_retain_drain_preserves_order_and_pending_entries() {
    let mut state = UiInputTimerState::default();
    let now = UiInputTimestamp::from_micros(10_000);

    state.arm_typeahead_expiration(UiNodeId::new(3), UiInputTimestamp::from_micros(0), 5);
    state.arm_typeahead_expiration(UiNodeId::new(1), UiInputTimestamp::from_micros(0), 20);
    state.arm_typeahead_expiration(UiNodeId::new(2), UiInputTimestamp::from_micros(0), 4);
    state.arm_submenu_hover_expiration(
        UiNodeId::new(3),
        "third",
        UiInputTimestamp::from_micros(0),
        5,
    );
    state.arm_submenu_hover_expiration(
        UiNodeId::new(1),
        "pending",
        UiInputTimestamp::from_micros(0),
        20,
    );
    state.arm_submenu_hover_expiration(
        UiNodeId::new(2),
        "second",
        UiInputTimestamp::from_micros(0),
        4,
    );

    assert_eq!(
        state.drain_expired_typeahead(now),
        vec![UiNodeId::new(2), UiNodeId::new(3)]
    );
    assert_eq!(
        state.drain_expired_submenu_hover(now),
        vec![
            (UiNodeId::new(2), "second".to_string()),
            (UiNodeId::new(3), "third".to_string()),
        ]
    );
    assert_eq!(
        state.typeahead_expiration(UiNodeId::new(1)),
        Some(UiInputTimestamp::from_micros(20_000))
    );
    assert_eq!(
        state.submenu_hover_option_id(UiNodeId::new(1)),
        Some("pending")
    );
}

#[test]
fn optimization_batch_20260826al_input_timer_uses_single_pass_retain_drains() {
    let source = include_str!("../../timers.rs");

    assert_eq!(source.matches(".retain(").count(), 4);
    assert!(!source.contains("for target in &expired"));
    assert!(!source.contains("for (target, _) in &expired"));
    assert!(source.matches("std::mem::take(").count() >= 3);
}

#[test]
fn optimization_batch_20260909_runtime274_toast_ref_arm_reuses_same_id_allocation_and_updates_deadline(
) {
    let target = UiNodeId::new(7);
    let mut state = UiInputTimerState::default();
    state.arm_toast_expiration(target, "save", UiInputTimestamp::from_micros(10), 40);
    let original_id = state.toast_id(target).expect("initial toast id");
    let original_storage = original_id.as_ptr();

    state.arm_toast_expiration_ref(target, "save", UiInputTimestamp::from_micros(20), 80);

    assert_eq!(state.toast_id(target), Some("save"));
    assert_eq!(
        state.toast_id(target).map(str::as_ptr),
        Some(original_storage),
        "same-id rearming should reuse the retained string"
    );
    assert_eq!(
        state.toast_expiration(target),
        Some(UiInputTimestamp::from_micros(80_020))
    );
}

#[test]
fn optimization_batch_20260909_runtime274_toast_ref_arm_replaces_changed_id_without_changing_deadline_semantics(
) {
    let target = UiNodeId::new(8);
    let mut state = UiInputTimerState::default();
    state.arm_toast_expiration_ref(target, "save", UiInputTimestamp::from_micros(10), 40);
    state.arm_toast_expiration_ref(target, "export", UiInputTimestamp::from_micros(20), 80);

    assert_eq!(state.toast_id(target), Some("export"));
    assert_eq!(
        state.toast_expiration(target),
        Some(UiInputTimestamp::from_micros(80_020))
    );
}

#[test]
fn runtime824_input_timer_drain_capacity_preserves_order_and_pending_entries() {
    let now = UiInputTimestamp::from_micros(10_000);
    let mut state = UiInputTimerState::default();
    for target in [3, 1, 2] {
        state.arm_typeahead_expiration(
            UiNodeId::new(target),
            UiInputTimestamp::from_micros(0),
            if target == 1 { 20 } else { 5 },
        );
        state.arm_submenu_hover_expiration(
            UiNodeId::new(target),
            format!("option-{target}"),
            UiInputTimestamp::from_micros(0),
            if target == 1 { 20 } else { 5 },
        );
        state.arm_tooltip_expiration(
            UiNodeId::new(target),
            format!("tooltip-{target}"),
            UiInputTimestamp::from_micros(0),
            if target == 1 { 20 } else { 5 },
        );
        state.arm_toast_expiration(
            UiNodeId::new(target),
            format!("toast-{target}"),
            UiInputTimestamp::from_micros(0),
            if target == 1 { 20 } else { 5 },
        );
    }

    let typeahead = state.drain_expired_typeahead(now);
    assert_eq!(typeahead, vec![UiNodeId::new(2), UiNodeId::new(3)]);
    assert!(typeahead.capacity() >= 3);
    let submenu = state.drain_expired_submenu_hover(now);
    assert_eq!(
        submenu,
        vec![
            (UiNodeId::new(2), "option-2".to_owned()),
            (UiNodeId::new(3), "option-3".to_owned()),
        ]
    );
    assert!(submenu.capacity() >= 3);
    let tooltips = state.drain_expired_tooltips(now);
    assert_eq!(
        tooltips,
        vec![
            (UiNodeId::new(2), "tooltip-2".to_owned()),
            (UiNodeId::new(3), "tooltip-3".to_owned()),
        ]
    );
    assert!(tooltips.capacity() >= 3);
    let toasts = state.drain_expired_toasts(now);
    assert_eq!(
        toasts,
        vec![
            (UiNodeId::new(2), "toast-2".to_owned()),
            (UiNodeId::new(3), "toast-3".to_owned()),
        ]
    );
    assert!(toasts.capacity() >= 3);

    assert_eq!(
        state.typeahead_expiration(UiNodeId::new(1)),
        Some(UiInputTimestamp::from_micros(20_000))
    );
    assert_eq!(
        state.submenu_hover_option_id(UiNodeId::new(1)),
        Some("option-1")
    );
    assert_eq!(state.tooltip_id(UiNodeId::new(1)), Some("tooltip-1"));
    assert_eq!(state.toast_id(UiNodeId::new(1)), Some("toast-1"));

    let no_expiry = state.drain_expired_typeahead(UiInputTimestamp::from_micros(10_001));
    assert!(no_expiry.is_empty());
    assert_eq!(no_expiry.capacity(), 0);
    black_box((typeahead, submenu, tooltips, toasts));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime824_input_timer_drain_capacity_release_benchmark() {
    const TIMERS: usize = 4_096;
    const SAMPLES: usize = 17;
    let mut legacy = Vec::with_capacity(SAMPLES);
    let mut optimized = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        if sample % 2 == 0 {
            legacy.push(measure_growth(TIMERS, false));
            optimized.push(measure_growth(TIMERS, true));
        } else {
            optimized.push(measure_growth(TIMERS, true));
            legacy.push(measure_growth(TIMERS, false));
        }
    }
    let legacy_p95 = nearest_rank(&legacy, 95);
    let optimized_p95 = nearest_rank(&optimized, 95);
    assert!(optimized_p95 <= legacy_p95);
    println!(
        "RUNTIME824_INPUT_TIMER_DRAIN_CAPACITY_BENCH_V1 timers={TIMERS} samples={SAMPLES} legacy_growth_events={} optimized_growth_events=0 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}",
        growth_events(TIMERS),
    );
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

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826al_input_timer_retain_drain_p95() {
    const TIMERS: usize = 16_384;
    let now = UiInputTimestamp::from_micros(10_000);
    let mut fixture = UiInputTimerState::default();
    for index in 0..TIMERS {
        let delay_ms = if index % 2 == 0 { 5 } else { 20 };
        fixture.arm_tooltip_expiration(
            UiNodeId::new(index as u64),
            format!("tooltip-{index}"),
            UiInputTimestamp::from_micros(0),
            delay_ms,
        );
    }

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(
                fixture.clone(),
                now,
                legacy_drain_expired_tooltips,
            ));
            optimized_ns.push(measure_ns(fixture.clone(), now, |state, now| {
                state.drain_expired_tooltips(now)
            }));
        } else {
            optimized_ns.push(measure_ns(fixture.clone(), now, |state, now| {
                state.drain_expired_tooltips(now)
            }));
            legacy_ns.push(measure_ns(
                fixture.clone(),
                now,
                legacy_drain_expired_tooltips,
            ));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "single-pass timer drain P95 must be at least 30% below clone-and-remove: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME77_INPUT_TIMER_RETAIN_DRAIN_BENCH_V1 timers={TIMERS} expired={} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_scans=1 legacy_removals={} optimized_retain_scans=1 optimized_payload_clones=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        TIMERS / 2,
        TIMERS / 2,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn legacy_drain_expired_tooltips(
    state: &mut UiInputTimerState,
    now: UiInputTimestamp,
) -> Vec<(UiNodeId, String)> {
    let expired = state
        .tooltip_expirations
        .iter()
        .filter_map(|(target, expiration)| {
            (expiration.deadline <= now).then(|| (*target, expiration.tooltip_id.clone()))
        })
        .collect::<Vec<_>>();
    for (target, _) in &expired {
        state.tooltip_expirations.remove(target);
    }
    expired
}

fn measure_ns(
    mut state: UiInputTimerState,
    now: UiInputTimestamp,
    operation: impl FnOnce(&mut UiInputTimerState, UiInputTimestamp) -> Vec<(UiNodeId, String)>,
) -> u128 {
    let started = Instant::now();
    let expired = operation(black_box(&mut state), black_box(now));
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(black_box(expired.len()), 8_192);
    assert_eq!(state.tooltip_expirations.len(), 8_192);
    elapsed
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
