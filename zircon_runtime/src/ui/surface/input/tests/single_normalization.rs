use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::dispatch::{
    UiAnalogInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiUserId,
};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn canonical_control_normalization_borrows_without_allocating() {
    assert!(matches!(
        normalized_control_name("gamepadleftx"),
        std::borrow::Cow::Borrowed("gamepadleftx")
    ));
    assert_eq!(
        normalized_control_name("G-a-m-e-p-a-d_L-e-f-t-X"),
        "gamepadleftx"
    );
}

#[test]
fn runtime77_analog_navigation_preserves_normalized_repeat_keys() {
    let mut input = UiSurfaceInputState::default();
    let mut analog = analog_event("G-a-m-e-p-a-d_L-e-f-t-X", 0.75, 1_000_000);

    assert_eq!(
        analog_navigation_decision(&mut input, &analog, analog.value),
        AnalogNavigationDecision::Navigate(UiNavigationEventKind::Right)
    );
    assert!(input
        .analog_navigation
        .contains_key("user7:gamepadleftx:Right"));

    analog.metadata.timestamp = UiInputTimestamp::from_micros(1_100_000);
    assert_eq!(
        analog_navigation_decision(&mut input, &analog, analog.value),
        AnalogNavigationDecision::Suppressed(UiNavigationEventKind::Right)
    );

    analog.value = 0.0;
    assert_eq!(
        analog_navigation_decision(&mut input, &analog, analog.value),
        AnalogNavigationDecision::Inactive
    );
    assert!(input.analog_navigation.is_empty());
}

#[test]
fn runtime77_analog_navigation_normalizes_control_once_per_event() {
    let source = include_str!("../analog_navigation.rs");
    let decision = bounded_function(
        source,
        "pub(super) fn analog_navigation_decision",
        "fn analog_navigation_kind",
    );
    let state_key = bounded_function(
        source,
        "fn analog_navigation_state_key",
        "enum AnalogNavigationAxis",
    );

    assert_eq!(
        source
            .matches("normalized_control_name(analog.control.as_str())")
            .count(),
        1
    );
    assert!(decision.contains("let normalized_control ="));
    assert!(decision.contains("normalized_control.as_ref()"));
    assert!(!state_key.contains("normalized_control_name"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime77_analog_navigation_single_normalization_p95() {
    const EVENTS: usize = 32_768;
    const CONTROL: &str = "G---a---m---e---p---a---d___L---e---f---t___S---t---i---c---k---X";

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(EVENTS, || legacy_control_signature(CONTROL)));
            optimized_ns.push(measure_ns(EVENTS, || optimized_control_signature(CONTROL)));
        } else {
            optimized_ns.push(measure_ns(EVENTS, || optimized_control_signature(CONTROL)));
            legacy_ns.push(measure_ns(EVENTS, || legacy_control_signature(CONTROL)));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(4),
        "single analog normalization P95 must be at least 20% below duplicate normalization: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME77_ANALOG_NAVIGATION_SINGLE_NORMALIZATION_BENCH_V1 events_per_sample={EVENTS} control_bytes={} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_normalizations_per_event=2 optimized_normalizations_per_event=1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        CONTROL.len(),
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn analog_event(control: &str, value: f32, monotonic_micros: u64) -> UiAnalogInputEvent {
    let mut metadata = UiInputEventMetadata::new(
        UiInputTimestamp::from_micros(monotonic_micros),
        UiInputSequence::new(monotonic_micros),
    );
    metadata.user_id = Some(UiUserId::new(7));
    UiAnalogInputEvent {
        metadata,
        control: control.to_string(),
        value,
    }
}

fn legacy_control_signature(control: &str) -> usize {
    let axis_normalized = normalized_control_name(control);
    let axis = analog_navigation_axis(axis_normalized.as_ref());
    let key_normalized = normalized_control_name(control);
    let key = format!("user7:{key_normalized}:Right");
    usize::from(axis.is_some()) + key.len()
}

fn optimized_control_signature(control: &str) -> usize {
    let normalized = normalized_control_name(control);
    let axis = analog_navigation_axis(normalized.as_ref());
    let key = format!("user7:{normalized}:Right");
    usize::from(axis.is_some()) + key.len()
}

fn measure_ns(iterations: usize, operation: impl Fn() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_function<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("function start")
        .split(end)
        .next()
        .expect("function end")
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
