use std::hint::black_box;
use std::time::{Duration, Instant};

use super::state_flags_summary;
use zircon_runtime_interface::ui::component::UiComponentState;

const SAMPLE_PAIRS: usize = 101;
const SUMMARIES_PER_SAMPLE: usize = 4_096;

#[test]
fn editor895_showcase_state_flag_stack_preserves_all_masks() {
    for mask in 0..=u8::MAX {
        let state = state_with_mask(mask);
        assert_eq!(
            state_flags_summary(&state),
            legacy_summary(&state),
            "mask={mask}"
        );
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn editor895_showcase_state_flag_stack_release_percentiles() {
    let state = state_with_mask(u8::MAX);
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&state, legacy_summary));
            optimized.push(measure(&state, state_flags_summary));
        } else {
            optimized.push(measure(&state, state_flags_summary));
            legacy.push(measure(&state, legacy_summary));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "EDITOR895_SHOWCASE_STATE_FLAG_STACK_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn state_with_mask(mask: u8) -> UiComponentState {
    let mut state = UiComponentState::new();
    state.flags.focused = mask & 1 != 0;
    state.flags.hovered = mask & 2 != 0;
    state.flags.pressed = mask & 4 != 0;
    state.flags.dragging = mask & 8 != 0;
    state.flags.popup_open = mask & 16 != 0;
    state.flags.expanded = mask & 32 != 0;
    state.flags.selected = mask & 64 != 0;
    state.flags.checked = mask & 128 != 0;
    state
}

fn legacy_summary(state: &UiComponentState) -> String {
    let flags = &state.flags;
    let mut states = Vec::with_capacity(8);
    if flags.focused {
        states.push("focused");
    }
    if flags.hovered {
        states.push("hovered");
    }
    if flags.pressed {
        states.push("pressed");
    }
    if flags.dragging {
        states.push("dragging");
    }
    if flags.popup_open {
        states.push("popup open");
    }
    if flags.expanded {
        states.push("expanded");
    }
    if flags.selected {
        states.push("selected");
    }
    if flags.checked {
        states.push("checked");
    }
    if states.is_empty() {
        "No value payload".to_string()
    } else {
        states.join(", ")
    }
}

fn measure(state: &UiComponentState, summary: fn(&UiComponentState) -> String) -> Duration {
    let started = Instant::now();
    let checksum = (0..SUMMARIES_PER_SAMPLE)
        .map(|_| black_box(summary(black_box(state))).len())
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
