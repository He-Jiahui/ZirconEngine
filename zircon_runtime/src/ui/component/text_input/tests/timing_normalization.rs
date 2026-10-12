use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentState, UiValue,
};

use super::{normalized_timing_matches, validation_timing, TextInputValidationTiming};

const VALIDATION_EVENTS_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime768_text_input_timing_normalization_preserves_aliases() {
    let descriptor = UiComponentDescriptor::new(
        "TextInputFixture",
        "Text Input Fixture",
        UiComponentCategory::Visual,
        "text-field",
    );

    let change = UiComponentState::new().with_value(
        "validation_timing",
        UiValue::String(" Value_Changed ".to_string()),
    );
    let blur = UiComponentState::new()
        .with_value("validation_timing", UiValue::Enum("FOCUS-OUT".to_string()));
    let commit = UiComponentState::new().with_value(
        "validation_timing",
        UiValue::String("unsupported".to_string()),
    );

    assert_eq!(
        validation_timing(&change, &descriptor),
        TextInputValidationTiming::Change
    );
    assert_eq!(
        validation_timing(&blur, &descriptor),
        TextInputValidationTiming::Blur
    );
    assert_eq!(
        validation_timing(&commit, &descriptor),
        TextInputValidationTiming::Commit
    );
    assert!(normalized_timing_matches(" value_changed ", "valuechanged"));
    assert!(normalized_timing_matches("FOCUS-OUT", "focusout"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime768_text_input_timing_normalization_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_normalization(VALIDATION_EVENTS_PER_SAMPLE, true));
            optimized_ns.push(measure_normalization(VALIDATION_EVENTS_PER_SAMPLE, false));
        } else {
            optimized_ns.push(measure_normalization(VALIDATION_EVENTS_PER_SAMPLE, false));
            legacy_ns.push(measure_normalization(VALIDATION_EVENTS_PER_SAMPLE, true));
        }
    }

    let legacy_normalization_allocations = VALIDATION_EVENTS_PER_SAMPLE;
    let optimized_normalization_allocations = 0;
    assert!(legacy_normalization_allocations > optimized_normalization_allocations);
    assert_eq!(optimized_normalization_allocations, 0);

    println!(
        "RUNTIME768_TEXT_INPUT_TIMING_NORMALIZATION_BENCH_V1 validation_events_per_sample={VALIDATION_EVENTS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_normalization_allocations={legacy_normalization_allocations} optimized_normalization_allocations={optimized_normalization_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_normalization(events: usize, allocate: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..events {
        if allocate {
            black_box(
                " Value_Changed "
                    .chars()
                    .filter(|ch| *ch != '_' && *ch != '-' && !ch.is_whitespace())
                    .flat_map(char::to_lowercase)
                    .collect::<String>(),
            );
        } else {
            black_box(normalized_timing_matches(" Value_Changed ", "valuechanged"));
        }
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
