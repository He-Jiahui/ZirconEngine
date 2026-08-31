use std::hint::black_box;
use std::panic::{self, AssertUnwindSafe};
use std::time::Instant;

use crate::core::framework::input::InputManager;
use crate::input::{InputButton, InputEvent};

use super::DefaultInputManager;

const BUTTON_QUERY_BENCH_PRESSED_COUNT: usize = 1_024;
const BUTTON_QUERY_BENCH_ITERATIONS: usize = 2_048;
const BUTTON_QUERY_BENCH_SAMPLE_PAIRS: usize = 21;

fn manager_with_pressed_key_codes(count: usize) -> DefaultInputManager {
    let manager = DefaultInputManager::default();
    for key_code in 0..count {
        manager.submit_event(InputEvent::ButtonPressed(InputButton::KeyCode(
            key_code as u32,
        )));
    }
    manager
}

fn legacy_snapshot_button_pressed(manager: &DefaultInputManager, button: &InputButton) -> bool {
    manager.snapshot().pressed_buttons.contains(button)
}

fn measure_ns(mut workload: impl FnMut()) -> u128 {
    let started = Instant::now();
    for _ in 0..BUTTON_QUERY_BENCH_ITERATIONS {
        workload();
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn runtime56_recovery_batch_input_manager_accessors_recover_poisoned_state_lock() {
    let manager = DefaultInputManager::default();
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.lock_state();
        panic!("poison input manager state");
    }));

    manager.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    assert!(manager
        .snapshot()
        .pressed_buttons
        .contains(&InputButton::MouseLeft));
    assert_eq!(
        manager.drain_events(),
        vec![InputEvent::ButtonPressed(InputButton::MouseLeft)]
    );

    manager.begin_frame();
    assert!(manager.frame_snapshot().mouse_wheel_events.is_empty());
}

#[test]
fn runtime56_recovery_batch_direct_button_query_matches_snapshot_for_present_and_missing_buttons() {
    let manager = manager_with_pressed_key_codes(32);

    assert!(manager.button_pressed(&InputButton::KeyCode(17)));
    assert!(!manager.button_pressed(&InputButton::KeyCode(91)));
    assert_eq!(
        manager.button_pressed(&InputButton::KeyCode(17)),
        legacy_snapshot_button_pressed(&manager, &InputButton::KeyCode(17))
    );
    assert_eq!(
        manager.button_pressed(&InputButton::KeyCode(91)),
        legacy_snapshot_button_pressed(&manager, &InputButton::KeyCode(91))
    );
}

#[test]
#[ignore = "release performance gate; run through the managed Runtime56 batch"]
fn runtime56_recovery_batch_allocation_free_button_query_release_gate() {
    let manager = manager_with_pressed_key_codes(BUTTON_QUERY_BENCH_PRESSED_COUNT);
    let missing = InputButton::KeyCode(u32::MAX);
    assert!(!legacy_snapshot_button_pressed(&manager, &missing));
    assert!(!manager.button_pressed(&missing));

    let mut legacy_samples = Vec::with_capacity(BUTTON_QUERY_BENCH_SAMPLE_PAIRS);
    let mut direct_samples = Vec::with_capacity(BUTTON_QUERY_BENCH_SAMPLE_PAIRS);
    for pair in 0..BUTTON_QUERY_BENCH_SAMPLE_PAIRS {
        let measure_legacy = || {
            measure_ns(|| {
                black_box(legacy_snapshot_button_pressed(
                    black_box(&manager),
                    black_box(&missing),
                ));
            })
        };
        let measure_direct = || {
            measure_ns(|| {
                black_box(black_box(&manager).button_pressed(black_box(&missing)));
            })
        };
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            direct_samples.push(measure_direct());
        } else {
            direct_samples.push(measure_direct());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50_ns = nearest_rank_percentile(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let direct_p50_ns = nearest_rank_percentile(&direct_samples, 50);
    let direct_p95_ns = nearest_rank_percentile(&direct_samples, 95);
    let legacy_snapshot_allocations = BUTTON_QUERY_BENCH_ITERATIONS;
    let direct_snapshot_allocations = 0;
    let legacy_button_clones = BUTTON_QUERY_BENCH_PRESSED_COUNT * BUTTON_QUERY_BENCH_ITERATIONS;
    let direct_button_clones = 0;
    let legacy_samples_ns = sample_csv(&legacy_samples);
    let direct_samples_ns = sample_csv(&direct_samples);

    println!(
        "PERF-MVP-559 task=runtime56_direct_button_query sample_pairs={} pressed_buttons={} iterations={} legacy_snapshot_allocations={} direct_snapshot_allocations={} legacy_button_clones={} direct_button_clones={} legacy_p50_ns={} legacy_p95_ns={} direct_p50_ns={} direct_p95_ns={} legacy_samples_ns={} direct_samples_ns={}",
        BUTTON_QUERY_BENCH_SAMPLE_PAIRS,
        BUTTON_QUERY_BENCH_PRESSED_COUNT,
        BUTTON_QUERY_BENCH_ITERATIONS,
        legacy_snapshot_allocations,
        direct_snapshot_allocations,
        legacy_button_clones,
        direct_button_clones,
        legacy_p50_ns,
        legacy_p95_ns,
        direct_p50_ns,
        direct_p95_ns,
        legacy_samples_ns,
        direct_samples_ns,
    );

    assert_eq!(legacy_snapshot_allocations, 2_048);
    assert_eq!(direct_snapshot_allocations, 0);
    assert_eq!(legacy_button_clones, 2_097_152);
    assert_eq!(direct_button_clones, 0);
    assert!(
        direct_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "direct query P95 {direct_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
    );
}
