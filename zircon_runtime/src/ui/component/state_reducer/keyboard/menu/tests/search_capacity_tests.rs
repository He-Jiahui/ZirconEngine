use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::{all_search_option_ids, collect_filtered_option_id_refs, MenuSearchOption};

const OPTION_VALUES_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime763_menu_search_projection_preserves_nested_ids_and_capacity() {
    let options = vec![search_option(
        "root",
        vec![search_option("child", Vec::new())],
    )];

    let flattened_ids = all_search_option_ids(&options);

    assert_eq!(flattened_ids, ["root", "child"]);
    assert!(flattened_ids.capacity() >= 2);

    let filter_value = UiValue::Array(vec![
        UiValue::String("root".to_string()),
        UiValue::Array(vec![UiValue::Enum("child".to_string())]),
        UiValue::Flags(vec!["leaf".to_string(), "root".to_string()]),
    ]);
    let mut filtered_ids = HashSet::new();
    collect_filtered_option_id_refs(&filter_value, &mut filtered_ids);

    assert_eq!(filtered_ids.len(), 3);
    assert!(filtered_ids.contains("root"));
    assert!(filtered_ids.contains("child"));
    assert!(filtered_ids.contains("leaf"));
    assert!(filtered_ids.capacity() >= 3);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime763_menu_search_projection_capacity_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_growth(OPTION_VALUES_PER_SAMPLE, 0));
            optimized_ns.push(measure_growth(
                OPTION_VALUES_PER_SAMPLE,
                OPTION_VALUES_PER_SAMPLE,
            ));
        } else {
            optimized_ns.push(measure_growth(
                OPTION_VALUES_PER_SAMPLE,
                OPTION_VALUES_PER_SAMPLE,
            ));
            legacy_ns.push(measure_growth(OPTION_VALUES_PER_SAMPLE, 0));
        }
    }

    let legacy_growth_events = growth_events(OPTION_VALUES_PER_SAMPLE, 0);
    let optimized_growth_events = growth_events(OPTION_VALUES_PER_SAMPLE, OPTION_VALUES_PER_SAMPLE);
    assert!(legacy_growth_events > optimized_growth_events);
    assert_eq!(optimized_growth_events, 0);

    println!(
        "RUNTIME763_MENU_SEARCH_PROJECTION_CAPACITY_BENCH_V1 direct_options_per_sample={OPTION_VALUES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn search_option(id: &str, children: Vec<MenuSearchOption>) -> MenuSearchOption {
    MenuSearchOption {
        id: id.to_string(),
        text: id.to_string(),
        top_level_index: 0,
        top_level_id: "root".to_string(),
        default_focus_candidate: true,
        children,
    }
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
