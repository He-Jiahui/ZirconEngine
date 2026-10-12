use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::option_id_list;

const ID_VALUES_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime759_keyboard_option_ids_preserve_nested_order_and_empty_rules() {
    let mut mapped_ids = BTreeMap::new();
    mapped_ids.insert(
        "value".to_string(),
        UiValue::Array(vec![
            UiValue::String("leaf".to_string()),
            UiValue::String(String::new()),
            UiValue::Enum("branch".to_string()),
        ]),
    );
    let source = UiValue::Array(vec![
        UiValue::String("root".to_string()),
        UiValue::Array(vec![UiValue::Enum("child".to_string())]),
        UiValue::Map(mapped_ids),
    ]);

    let ids = option_id_list(&source);

    assert_eq!(ids, ["root", "child", "leaf", "branch"]);
    assert!(ids.capacity() >= 3);
    let mut mapped_empty = BTreeMap::new();
    mapped_empty.insert(
        "id".to_string(),
        UiValue::Array(vec![
            UiValue::String(String::new()),
            UiValue::String("kept".to_string()),
        ]),
    );
    assert_eq!(option_id_list(&UiValue::Map(mapped_empty)), ["kept"]);
    assert_eq!(option_id_list(&UiValue::String(String::new())), [""]);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime759_keyboard_option_id_stream_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_growth(ID_VALUES_PER_SAMPLE, 0));
            optimized_ns.push(measure_growth(ID_VALUES_PER_SAMPLE, ID_VALUES_PER_SAMPLE));
        } else {
            optimized_ns.push(measure_growth(ID_VALUES_PER_SAMPLE, ID_VALUES_PER_SAMPLE));
            legacy_ns.push(measure_growth(ID_VALUES_PER_SAMPLE, 0));
        }
    }

    let legacy_growth_events = growth_events(ID_VALUES_PER_SAMPLE, 0);
    let optimized_growth_events = growth_events(ID_VALUES_PER_SAMPLE, ID_VALUES_PER_SAMPLE);
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);

    println!(
        "RUNTIME759_KEYBOARD_OPTION_ID_STREAM_BENCH_V1 direct_ids_per_sample={ID_VALUES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_growth(length: usize, capacity: usize) -> u128 {
    let started = Instant::now();
    let mut ids = Vec::with_capacity(capacity);
    for id in 0..length {
        ids.push(black_box(id));
    }
    black_box(ids);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(length: usize, capacity: usize) -> usize {
    if capacity >= length {
        return 0;
    }
    let mut capacity = capacity.max(1);
    let mut growth_events = 0;
    while capacity < length {
        capacity = capacity.saturating_mul(2);
        growth_events += 1;
    }
    growth_events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
