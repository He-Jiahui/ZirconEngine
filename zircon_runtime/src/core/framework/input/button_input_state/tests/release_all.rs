use std::cell::Cell;
use std::cmp::Ordering;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::ButtonInputState;

thread_local! {
    static COMPARISON_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CountedButton(usize);

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PanickingCloneButton(usize);

impl Clone for PanickingCloneButton {
    fn clone(&self) -> Self {
        assert_ne!(self.0, 2, "injected clone failure");
        Self(self.0)
    }
}

impl Ord for CountedButton {
    fn cmp(&self, other: &Self) -> Ordering {
        COMPARISON_COUNT.with(|count| count.set(count.get().saturating_add(1)));
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for CountedButton {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn runtime56_recovery_batch_release_all_preserves_sorted_results_and_frame_transitions() {
    let mut buttons = ButtonInputState::default();
    buttons.press(4);
    buttons.release(&4);
    buttons.press(3);
    buttons.press(1);
    buttons.press(2);

    let released = buttons.release_all();

    assert_eq!(released, vec![1, 2, 3]);
    for button in 1..=3 {
        assert!(!buttons.pressed(&button));
        assert!(buttons.just_pressed(&button));
        assert!(buttons.just_released(&button));
    }
    assert!(buttons.just_released(&4));
}

#[test]
fn runtime56_recovery_batch_release_all_clone_failure_preserves_the_held_state() {
    let mut buttons = ButtonInputState::from_pressed((0..4).map(PanickingCloneButton));

    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        buttons.release_all();
    }));

    assert!(failure.is_err());
    for button in 0..4 {
        assert!(buttons.pressed(&PanickingCloneButton(button)));
        assert!(!buttons.just_released(&PanickingCloneButton(button)));
    }
}

#[test]
fn runtime56_recovery_batch_release_all_transfers_the_pressed_tree_without_per_button_searches() {
    const BUTTON_COUNT: usize = 4_096;
    let mut baseline = counted_buttons(BUTTON_COUNT);
    let mut optimized = counted_buttons(BUTTON_COUNT);

    reset_comparisons();
    let baseline_released = legacy_release_all(&mut baseline);
    let baseline_comparisons = comparisons();

    reset_comparisons();
    let optimized_released = optimized.release_all();
    let optimized_comparisons = comparisons();

    assert_eq!(optimized_released, baseline_released);
    assert!(baseline_comparisons > BUTTON_COUNT);
    assert_eq!(
        optimized_comparisons, 0,
        "an empty release-transition tree should accept the pressed tree without key comparisons"
    );
}

#[test]
#[ignore = "Windows-native release benchmark evidence"]
fn runtime56_recovery_batch_release_all_bulk_tree_transfer_meets_the_p95_gate() {
    const BUTTON_COUNT: usize = 32_768;
    const SAMPLES: usize = 11;
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        if sample % 2 == 0 {
            baseline_samples.push(measure_baseline(BUTTON_COUNT));
            optimized_samples.push(measure_optimized(BUTTON_COUNT));
        } else {
            optimized_samples.push(measure_optimized(BUTTON_COUNT));
            baseline_samples.push(measure_baseline(BUTTON_COUNT));
        }
    }

    let baseline_p95 = percentile_95(&mut baseline_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_percent =
        100.0 * (1.0 - optimized_p95.as_secs_f64() / baseline_p95.as_secs_f64());

    println!(
        "RUNTIME56_BULK_BUTTON_RELEASE_BENCH_V1 buttons={BUTTON_COUNT} samples={SAMPLES} baseline_p95_ns={} optimized_p95_ns={} reduction_percent={reduction_percent:.2}",
        baseline_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );

    assert!(
        optimized_p95 <= baseline_p95.mul_f64(0.60),
        "bulk tree transfer must reduce release-all P95 by at least 40%: baseline={baseline_p95:?}, optimized={optimized_p95:?}"
    );
    assert!(
        optimized_p95 <= Duration::from_millis(50),
        "32,768-button release-all P95 must remain within 50 ms: {optimized_p95:?}"
    );
}

fn counted_buttons(count: usize) -> ButtonInputState<CountedButton> {
    ButtonInputState::from_pressed((0..count).map(CountedButton))
}

fn legacy_release_all<T>(buttons: &mut ButtonInputState<T>) -> Vec<T>
where
    T: Clone + Ord,
{
    let released = buttons.pressed.iter().cloned().collect::<Vec<_>>();
    for input in &released {
        buttons.release(input);
    }
    released
}

fn measure_baseline(count: usize) -> Duration {
    let mut buttons = ButtonInputState::from_pressed(0..count);
    let started = Instant::now();
    let released = legacy_release_all(&mut buttons);
    let elapsed = started.elapsed();
    black_box(released);
    elapsed
}

fn measure_optimized(count: usize) -> Duration {
    let mut buttons = ButtonInputState::from_pressed(0..count);
    let started = Instant::now();
    let released = buttons.release_all();
    let elapsed = started.elapsed();
    black_box(released);
    elapsed
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn reset_comparisons() {
    COMPARISON_COUNT.with(|count| count.set(0));
}

fn comparisons() -> usize {
    COMPARISON_COUNT.with(Cell::get)
}
