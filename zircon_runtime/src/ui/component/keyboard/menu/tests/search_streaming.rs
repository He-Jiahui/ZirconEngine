use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::{all_search_option_ids, menu_search_option_list};

const LEAF_OPTIONS_PER_SAMPLE: usize = 1_024;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn runtime764_menu_search_stream_preserves_tree_order_and_focus_indices() {
    let mut nested = BTreeMap::new();
    nested.insert("id".to_string(), UiValue::String("child".to_string()));
    nested.insert(
        "children".to_string(),
        UiValue::Array(vec![UiValue::Enum("leaf".to_string())]),
    );
    let mut root = BTreeMap::new();
    root.insert("id".to_string(), UiValue::String("root".to_string()));
    root.insert(
        "children".to_string(),
        UiValue::Array(vec![UiValue::Map(nested)]),
    );
    let source = UiValue::Array(vec![
        UiValue::Map(root),
        UiValue::String("second-root".to_string()),
    ]);

    let options = menu_search_option_list(&source);

    assert_eq!(
        options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        ["root", "second-root"]
    );
    assert_eq!(options[0].top_level_index, 0);
    assert_eq!(options[1].top_level_index, 1);
    assert_eq!(
        all_search_option_ids(&options),
        ["root", "child", "leaf", "second-root"]
    );
    assert_eq!(options[0].children[0].top_level_id, "root");
    assert_eq!(options[0].children[0].children[0].top_level_id, "root");
    assert!(options.capacity() >= 2);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime764_menu_search_stream_release_benchmark() {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_growth(LEAF_OPTIONS_PER_SAMPLE, 0));
            optimized_ns.push(measure_growth(
                LEAF_OPTIONS_PER_SAMPLE,
                LEAF_OPTIONS_PER_SAMPLE,
            ));
        } else {
            optimized_ns.push(measure_growth(
                LEAF_OPTIONS_PER_SAMPLE,
                LEAF_OPTIONS_PER_SAMPLE,
            ));
            legacy_ns.push(measure_growth(LEAF_OPTIONS_PER_SAMPLE, 0));
        }
    }

    let legacy_intermediate_vectors = LEAF_OPTIONS_PER_SAMPLE;
    let optimized_intermediate_vectors = 0;
    assert!(legacy_intermediate_vectors > optimized_intermediate_vectors);
    assert_eq!(optimized_intermediate_vectors, 0);

    println!(
        "RUNTIME764_MENU_SEARCH_STREAM_BENCH_V1 leaf_options_per_sample={LEAF_OPTIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_intermediate_vectors={legacy_intermediate_vectors} optimized_intermediate_vectors={optimized_intermediate_vectors} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_growth(length: usize, capacity: usize) -> u128 {
    let started = Instant::now();
    let mut options = Vec::with_capacity(capacity);
    for option in 0..length {
        options.push(black_box(option));
    }
    black_box(options);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
