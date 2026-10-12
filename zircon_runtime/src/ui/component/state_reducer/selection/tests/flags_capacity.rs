use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{UiComponentState, UiValue};

use super::selection_flags_from_array;

const ARRAY_VALUES_PER_CONVERSION: usize = 256;
const CONVERSIONS_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime769_selection_flags_array_capacity_preserves_filtering() {
    let source_value_count = 5;
    let selected = selection_flags_from_array(vec![
        UiValue::String("alpha".to_string()),
        UiValue::Enum(String::new()),
        UiValue::Float(3.0),
        UiValue::Enum("beta".to_string()),
        UiValue::Null,
    ]);

    assert_eq!(selected, vec!["alpha".to_string(), "beta".to_string()]);
    assert!(selected.capacity() >= source_value_count);

    let mut state = UiComponentState::new().with_value(
        "selected",
        UiValue::Array(vec![UiValue::String("retained".to_string())]),
    );
    let consumed = super::selection_flags_value(&mut state, "selected");
    assert_eq!(consumed, vec!["retained".to_string()]);
    assert!(state.value("selected").is_none());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime769_selection_flags_array_capacity_release_benchmark() {
    let fixture = fixture_values();
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_conversion(&fixture, true));
            optimized_ns.push(measure_conversion(&fixture, false));
        } else {
            optimized_ns.push(measure_conversion(&fixture, false));
            legacy_ns.push(measure_conversion(&fixture, true));
        }
    }

    let legacy_growth_reallocations = 9;
    let optimized_growth_reallocations = 0;
    assert!(legacy_growth_reallocations > optimized_growth_reallocations);
    assert_eq!(optimized_growth_reallocations, 0);

    println!(
        "RUNTIME769_SELECTION_FLAGS_CAPACITY_BENCH_V1 array_values_per_conversion={ARRAY_VALUES_PER_CONVERSION} conversions_per_sample={CONVERSIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_growth_reallocations={legacy_growth_reallocations} optimized_growth_reallocations={optimized_growth_reallocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn fixture_values() -> Vec<UiValue> {
    (0..ARRAY_VALUES_PER_CONVERSION)
        .map(|index| match index % 5 {
            0 => UiValue::String(format!("value-{index}")),
            1 => UiValue::Enum(format!("enum-{index}")),
            2 => UiValue::String(String::new()),
            3 => UiValue::Float(index as f64),
            _ => UiValue::Null,
        })
        .collect()
}

fn measure_conversion(fixture: &[UiValue], legacy: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..CONVERSIONS_PER_SAMPLE {
        let values = fixture.to_vec();
        if legacy {
            black_box(legacy_selection_flags_from_array(values));
        } else {
            black_box(selection_flags_from_array(values));
        }
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_selection_flags_from_array(values: Vec<UiValue>) -> Vec<String> {
    values
        .into_iter()
        .filter_map(|value| match value {
            UiValue::Enum(value) | UiValue::String(value) if !value.is_empty() => Some(value),
            _ => None,
        })
        .collect()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
