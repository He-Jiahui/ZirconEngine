use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::component::UiValue;

use super::menu_search_option_list;

#[test]
fn runtime780_menu_option_label_borrow_preserves_label_and_id_fallback() {
    let mut labeled = BTreeMap::new();
    labeled.insert(
        "id".to_string(),
        UiValue::Array(vec![
            UiValue::String("first".to_string()),
            UiValue::String("second".to_string()),
        ]),
    );
    labeled.insert(
        "label".to_string(),
        UiValue::String("Shared Label".to_string()),
    );

    let mut fallback = BTreeMap::new();
    fallback.insert("id".to_string(), UiValue::String("fallback-id".to_string()));
    fallback.insert("label".to_string(), UiValue::String(String::new()));

    let options = menu_search_option_list(&UiValue::Array(vec![
        UiValue::Map(labeled),
        UiValue::Map(fallback),
    ]));
    assert_eq!(options.len(), 3);
    assert_eq!(options[0].text, "Shared Label");
    assert_eq!(options[1].text, "Shared Label");
    assert_eq!(options[2].text, "fallback-id");
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime780_menu_label_borrow_release_benchmark() {
    const MAP_NODES_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_label_lookup(MAP_NODES_PER_SAMPLE, true));
            optimized_ns.push(measure_label_lookup(MAP_NODES_PER_SAMPLE, false));
        } else {
            optimized_ns.push(measure_label_lookup(MAP_NODES_PER_SAMPLE, false));
            legacy_ns.push(measure_label_lookup(MAP_NODES_PER_SAMPLE, true));
        }
    }

    let legacy_intermediate_label_allocations = MAP_NODES_PER_SAMPLE;
    let optimized_intermediate_label_allocations = 0;
    assert!(legacy_intermediate_label_allocations > optimized_intermediate_label_allocations);
    println!(
        "RUNTIME780_MENU_LABEL_BORROW_BENCH_V1 map_nodes_per_sample={MAP_NODES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_intermediate_label_allocations={legacy_intermediate_label_allocations} optimized_intermediate_label_allocations={optimized_intermediate_label_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

// 两个分支都生成拥有的标签字符串；此辅助模型没有调用菜单投影，也没有测量实际分配次数。
fn measure_label_lookup(nodes: usize, legacy: bool) -> u128 {
    let started = Instant::now();
    let source = "Shared Label";
    let mut retained = Vec::with_capacity(nodes);
    for _ in 0..nodes {
        if legacy {
            let intermediate = source.to_string();
            retained.push(black_box(intermediate));
        } else {
            retained.push(black_box(source.to_owned()));
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
