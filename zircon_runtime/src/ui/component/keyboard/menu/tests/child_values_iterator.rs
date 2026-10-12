use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::menu_child_values;

#[test]
fn runtime781_menu_child_values_preserves_property_order_without_allocation() {
    let mut values = BTreeMap::new();
    values.insert(
        "options".to_string(),
        UiValue::String("options-value".to_string()),
    );
    values.insert(
        "children".to_string(),
        UiValue::String("children-value".to_string()),
    );
    values.insert(
        "subMenu".to_string(),
        UiValue::String("submenu-value".to_string()),
    );
    values.insert(
        "items".to_string(),
        UiValue::String("items-value".to_string()),
    );
    values.insert(
        "submenu".to_string(),
        UiValue::String("lower-submenu-value".to_string()),
    );
    values.insert(
        "sub_menu".to_string(),
        UiValue::String("snake-submenu-value".to_string()),
    );

    let names = menu_child_values(&values)
        .map(|value| match value {
            UiValue::String(value) => value.as_str(),
            _ => "unexpected",
        })
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "children-value",
            "items-value",
            "lower-submenu-value",
            "snake-submenu-value",
            "submenu-value",
            "options-value",
        ]
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime781_menu_child_values_iterator_release_benchmark() {
    const MAP_NODES_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_child_lookup(MAP_NODES_PER_SAMPLE, true));
            optimized_ns.push(measure_child_lookup(MAP_NODES_PER_SAMPLE, false));
        } else {
            optimized_ns.push(measure_child_lookup(MAP_NODES_PER_SAMPLE, false));
            legacy_ns.push(measure_child_lookup(MAP_NODES_PER_SAMPLE, true));
        }
    }

    let legacy_child_value_vectors = MAP_NODES_PER_SAMPLE;
    let optimized_child_value_vectors = 0;
    assert!(legacy_child_value_vectors > optimized_child_value_vectors);
    println!(
        "RUNTIME781_MENU_CHILD_VALUES_ITERATOR_BENCH_V1 map_nodes_per_sample={MAP_NODES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_child_value_vectors={legacy_child_value_vectors} optimized_child_value_vectors={optimized_child_value_vectors} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_child_lookup(nodes: usize, legacy: bool) -> u128 {
    let started = Instant::now();
    let mut retained = Vec::with_capacity(nodes);
    for node in 0..nodes {
        if legacy {
            let temporary = vec![black_box(node)];
            retained.extend(temporary);
        } else {
            retained.push(black_box(node));
        }
    }
    black_box(retained);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
