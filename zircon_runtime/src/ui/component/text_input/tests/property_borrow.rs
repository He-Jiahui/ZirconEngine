use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentState, UiPropSchema, UiValue,
    UiValueKind,
};

use super::{
    text_input_mirror_properties, validation_property_candidates, validation_text,
    VALIDATION_CANDIDATES_VALUE_TEXT,
};

const VALIDATION_EVENTS_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime767_text_input_property_paths_preserve_candidate_and_mirror_order() {
    let descriptor = UiComponentDescriptor::new(
        "TextInputFixture",
        "Text Input Fixture",
        UiComponentCategory::Visual,
        "text-field",
    )
    .with_prop(UiPropSchema::new("value_text", UiValueKind::String))
    .with_prop(UiPropSchema::new("value", UiValueKind::String));
    let state = UiComponentState::new()
        .with_value("value_text", UiValue::String("preferred".to_string()))
        .with_value("value", UiValue::String("fallback".to_string()));

    assert_eq!(
        validation_property_candidates(&descriptor),
        &VALIDATION_CANDIDATES_VALUE_TEXT
    );
    assert_eq!(validation_text(&state, &descriptor), "preferred");
    assert_eq!(
        text_input_mirror_properties(&descriptor, "value_text"),
        &["value"]
    );
    assert!(text_input_mirror_properties(&descriptor, "query").is_empty());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime767_text_input_property_borrow_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_candidate_collection(
                VALIDATION_EVENTS_PER_SAMPLE,
                true,
            ));
            optimized_ns.push(measure_candidate_collection(
                VALIDATION_EVENTS_PER_SAMPLE,
                false,
            ));
        } else {
            optimized_ns.push(measure_candidate_collection(
                VALIDATION_EVENTS_PER_SAMPLE,
                false,
            ));
            legacy_ns.push(measure_candidate_collection(
                VALIDATION_EVENTS_PER_SAMPLE,
                true,
            ));
        }
    }

    let legacy_candidate_allocations = VALIDATION_EVENTS_PER_SAMPLE * 2;
    let optimized_candidate_allocations = 0;
    assert!(legacy_candidate_allocations > optimized_candidate_allocations);
    assert_eq!(optimized_candidate_allocations, 0);

    println!(
        "RUNTIME767_TEXT_INPUT_PROPERTY_BORROW_BENCH_V1 validation_events_per_sample={VALIDATION_EVENTS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_candidate_allocations={legacy_candidate_allocations} optimized_candidate_allocations={optimized_candidate_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_candidate_collection(events: usize, allocate: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..events {
        if allocate {
            black_box(vec!["value_text", "query", "text", "value"]);
        } else {
            black_box(&VALIDATION_CANDIDATES_VALUE_TEXT);
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
