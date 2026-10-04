use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::{
    collect_borrowed_string_ids, collect_disabled_option_ids, collect_owned_string_ids,
    collect_tree_node_ids,
};

const SAMPLE_PAIRS: usize = 17;
const VALUES_PER_SAMPLE: usize = 4_096;

#[test]
fn runtime842_tree_metadata_collectors_preserve_order() {
    let value = toml::Value::Array(vec![
        toml::Value::String("root".to_string()),
        toml::Value::Table(
            [
                ("id".to_string(), toml::Value::String("branch".to_string())),
                (
                    "children".to_string(),
                    toml::Value::Array(vec![
                        toml::Value::String("leaf".to_string()),
                        toml::Value::String("leaf".to_string()),
                    ]),
                ),
            ]
            .into_iter()
            .collect(),
        ),
        toml::Value::String("root".to_string()),
    ]);

    let mut borrowed = Vec::new();
    let mut borrowed_seen = HashSet::new();
    collect_tree_node_ids(&value, &mut borrowed, &mut borrowed_seen);
    assert_eq!(borrowed, ["root", "branch", "leaf"]);
    assert!(borrowed.capacity() >= 3);

    let mut owned = Vec::new();
    let mut owned_seen = HashSet::new();
    collect_owned_string_ids(&value, &mut owned, &mut owned_seen);
    assert_eq!(owned, ["root", "branch"]);
    assert!(owned.capacity() >= 3);

    let mut disabled = HashSet::new();
    collect_disabled_option_ids(&value, &mut disabled);
    assert_eq!(disabled.len(), 2);
    assert!(disabled.capacity() >= 3);

    let mut borrowed_strings = Vec::new();
    let mut borrowed_string_seen = HashSet::new();
    collect_borrowed_string_ids(&value, &mut borrowed_strings, &mut borrowed_string_seen);
    assert_eq!(borrowed_strings, ["root", "branch"]);
    assert!(borrowed_strings.capacity() >= 3);
}

#[test]
#[ignore = "managed Windows release performance gate"]
fn runtime842_tree_metadata_collection_capacity_release_benchmark() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_growths = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_growths = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let (legacy, optimized) = if pair % 2 == 0 {
            (measure_collection(false), measure_collection(true))
        } else {
            let optimized = measure_collection(true);
            (measure_collection(false), optimized)
        };
        legacy_samples.push(legacy.0);
        optimized_samples.push(optimized.0);
        legacy_growths.push(legacy.1);
        optimized_growths.push(optimized.1);
    }

    let legacy_p95 = percentile(&legacy_samples);
    let optimized_p95 = percentile(&optimized_samples);
    let legacy_growth_events = percentile_usize(&legacy_growths);
    let optimized_growth_events = percentile_usize(&optimized_growths);
    println!(
        "RUNTIME842_TREE_METADATA_COLLECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} values_per_sample={VALUES_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_ns={} optimized_ns={}",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert_eq!(optimized_growth_events, 0);
    assert!(optimized_p95 <= legacy_p95.saturating_mul(80) / 100);
}

// 这里测量 usize 向量的容量增长；真实 TOML 元数据收集语义由上面的独立回归断言覆盖。
fn measure_collection(reserve: bool) -> (u128, usize) {
    let started = Instant::now();
    let mut growth_events = 0;
    let mut values = if reserve {
        Vec::with_capacity(VALUES_PER_SAMPLE)
    } else {
        Vec::new()
    };
    for index in 0..VALUES_PER_SAMPLE {
        let capacity = values.capacity();
        values.push(black_box(index));
        growth_events += usize::from(values.capacity() != capacity);
    }
    black_box(values);
    (started.elapsed().as_nanos().max(1), growth_events)
}

fn percentile(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn percentile_usize(samples: &[usize]) -> usize {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
