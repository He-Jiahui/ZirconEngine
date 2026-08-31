use std::hint::black_box;
use std::time::Instant;

const BUTTON_COUNT: usize = 32;
const CHECKS_PER_SAMPLE: usize = 100_000;
const SAMPLE_PAIRS: usize = 31;

#[derive(Clone, Copy)]
struct ButtonState {
    pressed: bool,
    just_pressed: bool,
    just_released: bool,
}

#[inline(never)]
fn legacy_button_state(states: &[ButtonState]) -> (bool, bool, bool) {
    let all_pressed = black_box(states).iter().all(|state| state.pressed);
    let any_just_pressed = black_box(states).iter().any(|state| state.just_pressed);
    let any_just_released = black_box(states).iter().any(|state| state.just_released);
    (all_pressed, any_just_pressed, any_just_released)
}

#[inline(never)]
fn optimized_button_state(states: &[ButtonState]) -> (bool, bool, bool) {
    let mut all_pressed = true;
    let mut any_just_pressed = false;
    let mut any_just_released = false;
    for state in black_box(states) {
        if all_pressed {
            all_pressed = state.pressed;
        }
        if !any_just_pressed {
            any_just_pressed = state.just_pressed;
        }
        if !any_just_released {
            any_just_released = state.just_released;
        }
        if !all_pressed && any_just_pressed && any_just_released {
            break;
        }
    }
    (all_pressed, any_just_pressed, any_just_released)
}

fn measure(states: &[ButtonState], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0_usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        let (pressed, just_pressed, just_released) = if optimized {
            optimized_button_state(states)
        } else {
            legacy_button_state(states)
        };
        evidence = evidence
            .wrapping_add(pressed as usize + just_pressed as usize + just_released as usize);
    }
    black_box(evidence);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
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
fn optimization_batch_20260829ca_runtime354_single_pass_button_state_preserves_results() {
    let cases = [
        Vec::new(),
        vec![ButtonState {
            pressed: true,
            just_pressed: false,
            just_released: false,
        }],
        vec![
            ButtonState {
                pressed: true,
                just_pressed: false,
                just_released: false,
            },
            ButtonState {
                pressed: false,
                just_pressed: true,
                just_released: true,
            },
        ],
    ];
    for states in cases {
        assert_eq!(
            optimized_button_state(&states),
            legacy_button_state(&states)
        );
    }
}

#[test]
fn optimization_batch_20260829ca_runtime354_production_scans_button_state_once() {
    let source = include_str!("../action_evaluator.rs");
    let function = source
        .split_once("fn evaluate_with_workspace(")
        .expect("workspace evaluator")
        .1
        .split_once("fn action_context_is_active(")
        .expect("context boundary")
        .0;
    assert_eq!(
        function.matches("for button in &binding.buttons").count(),
        1
    );
    assert!(!function.contains(".all(|button| frame.buttons.pressed"));
    assert!(!function.contains(".any(|button| frame.buttons.just_pressed"));
    assert!(!function.contains(".any(|button| frame.buttons.just_released"));
}

#[test]
#[ignore = "managed performance gate"]
fn optimization_batch_20260829ca_runtime354_button_state_scan_benchmark() {
    let mut states = vec![
        ButtonState {
            pressed: true,
            just_pressed: false,
            just_released: false,
        };
        BUTTON_COUNT
    ];
    let last = states.last_mut().expect("benchmark buttons");
    last.just_pressed = true;
    last.just_released = true;

    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&states, false));
            candidate.push(measure(&states, true));
        } else {
            candidate.push(measure(&states, true));
            baseline.push(measure(&states, false));
        }
    }
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME354_BUTTON_STATE_SINGLE_PASS_BENCH_V1 baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_samples_ns={} candidate_samples_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns * 100 <= baseline_p95_ns * 70);
}
