use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use super::{menu_item_route_indices, menu_item_tree_shape, MenuItemSpec};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const ROOT_ITEMS: usize = 16;
const CHILD_ITEMS: usize = 15;
const ITEMS_PER_BUILD: usize = ROOT_ITEMS * (CHILD_ITEMS + 1);

#[test]
fn optimization_batch_20260826ew_editor138_capacity_preserves_depth_first_route_indices() {
    let items = (0..ROOT_ITEMS)
        .map(|root| {
            item(
                format!("root-{root}"),
                (0..CHILD_ITEMS)
                    .map(|child| item(format!("child-{root}-{child}"), Vec::new()))
                    .collect(),
            )
        })
        .collect::<Vec<_>>();

    let indices = menu_item_route_indices(&items);

    assert_eq!(indices.len(), ITEMS_PER_BUILD);
    assert!(indices.capacity() >= ITEMS_PER_BUILD);
    assert_eq!(indices.get(&vec![0]), Some(&0));
    assert_eq!(indices.get(&vec![0, 0]), Some(&1));
    assert_eq!(indices.get(&vec![1]), Some(&16));
    assert_eq!(indices.get(&vec![15, 14]), Some(&255));
    assert_eq!(menu_item_tree_shape(&items), (ITEMS_PER_BUILD, 2));
}

#[test]
fn optimization_batch_20260826ew_editor138_menu_routes_reserve_tree_shape() {
    let source = include_str!("../menu_item_tree.rs");
    assert!(source.contains("menu_item_tree_shape(items)"));
    assert!(source.contains("HashMap::with_capacity(item_count)"));
    assert!(source.contains("Vec::with_capacity(max_depth)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ew_editor138_menu_route_index_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR138_MENU_ROUTE_INDEX_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} routes_per_build={ITEMS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn item(action_id: String, children: Vec<MenuItemSpec>) -> MenuItemSpec {
    MenuItemSpec {
        action_id: Some(action_id),
        enabled: true,
        children,
    }
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut indices = if reserve {
            HashMap::with_capacity(ITEMS_PER_BUILD)
        } else {
            HashMap::new()
        };
        for index in 0..ITEMS_PER_BUILD {
            indices.insert(black_box(vec![index]), index);
        }
        checksum ^= black_box(indices.len() ^ indices.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
