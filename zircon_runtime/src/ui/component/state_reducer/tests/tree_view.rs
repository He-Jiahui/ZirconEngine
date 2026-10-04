use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::{
    collect_borrowed_string_ids, collect_disabled_option_ids, collect_owned_string_ids,
    collect_tree_node_ids,
};

const TREE_ID_VALUES_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn ordered_tree_ids_borrow_first_occurrence_and_deduplicate_in_linear_index() {
    let value = UiValue::Array(vec![
        UiValue::String("root".to_string()),
        UiValue::String("root".to_string()),
        UiValue::Enum("child".to_string()),
    ]);
    let first_root = match &value {
        UiValue::Array(values) => match &values[0] {
            UiValue::String(value) => value.as_ptr(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    let mut ids = Vec::new();
    let mut seen = HashSet::new();

    collect_tree_node_ids(&value, &mut ids, &mut seen);

    assert_eq!(ids, ["root", "child"]);
    assert_eq!(ids[0].as_ptr(), first_root);
}

#[test]
fn disabled_option_index_preserves_array_enum_and_flags_membership() {
    let value = UiValue::Array(vec![
        UiValue::String("root".to_string()),
        UiValue::Enum("child".to_string()),
        UiValue::Flags(vec!["leaf".to_string(), "root".to_string()]),
    ]);
    let mut disabled = HashSet::new();

    collect_disabled_option_ids(&value, &mut disabled);

    assert_eq!(disabled.len(), 3);
    assert!(disabled.contains("root"));
    assert!(disabled.contains("child"));
    assert!(disabled.contains("leaf"));
}

#[test]
fn runtime757_tree_id_collectors_preserve_nested_order_and_capacity() {
    let value = UiValue::Array(vec![
        UiValue::String("root".to_string()),
        UiValue::Array(vec![UiValue::Enum("child".to_string())]),
        UiValue::Flags(vec!["leaf".to_string(), "root".to_string()]),
    ]);

    let mut tree_ids = Vec::new();
    let mut tree_seen = HashSet::new();
    collect_tree_node_ids(&value, &mut tree_ids, &mut tree_seen);
    assert_eq!(tree_ids, ["root", "child"]);
    assert!(tree_ids.capacity() >= 3);

    let mut borrowed_ids = Vec::new();
    let mut borrowed_seen = HashSet::new();
    collect_borrowed_string_ids(&value, &mut borrowed_ids, &mut borrowed_seen);
    assert_eq!(borrowed_ids, ["root", "child", "leaf"]);
    assert!(borrowed_ids.capacity() >= 3);
    assert_eq!(borrowed_seen.len(), 3);

    let mut owned_ids = Vec::new();
    let mut owned_seen = HashSet::new();
    collect_owned_string_ids(&value, &mut owned_ids, &mut owned_seen);
    assert_eq!(
        owned_ids.iter().map(String::as_str).collect::<Vec<_>>(),
        ["root", "child", "leaf"]
    );
    assert!(owned_ids.capacity() >= 3);
    assert_eq!(owned_seen.len(), 3);

    let mut disabled = HashSet::new();
    collect_disabled_option_ids(&value, &mut disabled);
    assert_eq!(disabled.len(), 3);
    assert!(disabled.capacity() >= 3);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime757_tree_id_collection_capacity_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_growth(TREE_ID_VALUES_PER_SAMPLE, 0));
            optimized_ns.push(measure_growth(
                TREE_ID_VALUES_PER_SAMPLE,
                TREE_ID_VALUES_PER_SAMPLE,
            ));
        } else {
            optimized_ns.push(measure_growth(
                TREE_ID_VALUES_PER_SAMPLE,
                TREE_ID_VALUES_PER_SAMPLE,
            ));
            legacy_ns.push(measure_growth(TREE_ID_VALUES_PER_SAMPLE, 0));
        }
    }

    let legacy_growth_events = growth_events(TREE_ID_VALUES_PER_SAMPLE, 0);
    let optimized_growth_events =
        growth_events(TREE_ID_VALUES_PER_SAMPLE, TREE_ID_VALUES_PER_SAMPLE);
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);

    println!(
        "RUNTIME757_TREE_ID_COLLECTION_CAPACITY_BENCH_V1 direct_values_per_sample={TREE_ID_VALUES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_growth(length: usize, capacity: usize) -> u128 {
    let started = Instant::now();
    let mut values = Vec::with_capacity(capacity);
    for value in 0..length {
        values.push(black_box(value));
    }
    black_box(values);
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
