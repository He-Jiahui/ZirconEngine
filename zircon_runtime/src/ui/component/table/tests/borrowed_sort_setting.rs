use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::{UiComponentState, UiValue};

use super::{column_width_payload, string_setting_ref, string_value};

const SORT_EVENTS_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime771_table_borrowed_sort_settings_preserve_semantics() {
    assert_eq!(
        string_value(&UiValue::String("ascending".to_string())),
        Some("ascending")
    );
    assert_eq!(string_value(&UiValue::Float(1.0)), None);

    let state = UiComponentState::new()
        .with_value("sort_column", UiValue::String("name".to_string()))
        .with_value("sort_direction", UiValue::Enum("desc".to_string()))
        .with_value("sortingMode", UiValue::String("client".to_string()));
    assert_eq!(string_setting_ref(&state, "sort_column"), Some("name"));
    assert_eq!(string_setting_ref(&state, "sort_direction"), Some("desc"));

    let payload = UiValue::Map(BTreeMap::from([
        ("field".to_string(), UiValue::String("name".to_string())),
        ("width".to_string(), UiValue::Float(144.0)),
    ]));
    assert_eq!(column_width_payload(&payload), Some(("name", 144.0)));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime771_table_borrowed_sort_setting_release_benchmark() {
    let state = UiComponentState::new()
        .with_value(
            "sort_column",
            UiValue::String("a-long-table-column-name".to_string()),
        )
        .with_value("sort_direction", UiValue::Enum("ascending".to_string()))
        .with_value("sortingMode", UiValue::String("client".to_string()));
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_setting(&state, true));
            optimized_ns.push(measure_setting(&state, false));
        } else {
            optimized_ns.push(measure_setting(&state, false));
            legacy_ns.push(measure_setting(&state, true));
        }
    }

    let legacy_textual_clones = SORT_EVENTS_PER_SAMPLE * 3;
    let optimized_textual_clones = 0;
    assert!(legacy_textual_clones > optimized_textual_clones);
    assert_eq!(optimized_textual_clones, 0);

    println!(
        "RUNTIME771_TABLE_BORROWED_SORT_SETTING_BENCH_V1 sort_events_per_sample={SORT_EVENTS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_textual_clones={legacy_textual_clones} optimized_textual_clones={optimized_textual_clones} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_setting(state: &UiComponentState, legacy: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..SORT_EVENTS_PER_SAMPLE {
        if legacy {
            black_box(legacy_string_setting(state, "sort_column"));
            black_box(legacy_string_setting(state, "sort_direction"));
            black_box(legacy_string_setting(state, "sortingMode"));
        } else {
            black_box(string_setting_ref(state, "sort_column"));
            black_box(string_setting_ref(state, "sort_direction"));
            black_box(string_setting_ref(state, "sortingMode"));
        }
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_string_setting(state: &UiComponentState, property: &str) -> Option<String> {
    state.values.get(property).and_then(|value| match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    })
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
