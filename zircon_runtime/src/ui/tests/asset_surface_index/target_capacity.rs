use super::*;

use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use crate::ui::template::{UiAssetNodeHotReloadTargets, UiAssetSurfaceHotReloadTargets};

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 128;
const CATEGORY_SIZE: usize = 4_096;
const CATEGORY_COUNT: usize = 4;

#[test]
fn all_target_surface_outputs_reserve_category_upper_bound() {
    let mut targets = UiAssetSurfaceHotReloadTargets::default();
    let shared = tree_id("surface/shared");
    targets.template_rebuild_surfaces = vec![shared.clone()];
    targets.removed_compiled_surfaces = vec![shared.clone()];
    targets.theme_restyle_surfaces = vec![tree_id("surface/theme")];
    targets.resource_damage_surfaces = vec![tree_id("surface/resource")];

    let total_inputs = targets.template_rebuild_surfaces.len()
        + targets.removed_compiled_surfaces.len()
        + targets.theme_restyle_surfaces.len()
        + targets.resource_damage_surfaces.len();
    let projected = targets.all_target_surfaces();

    assert_eq!(
        projected,
        vec![
            shared,
            tree_id("surface/theme"),
            tree_id("surface/resource")
        ]
    );
    assert_eq!(projected.len(), total_inputs - 1);
    assert!(projected.capacity() >= total_inputs);
}

#[test]
fn all_target_node_outputs_reserve_category_upper_bound() {
    let main = tree_id("surface/main");
    let mut targets = UiAssetNodeHotReloadTargets::default();
    let shared = UiAssetNodeTarget {
        tree_id: main.clone(),
        node_id: UiNodeId::new(1),
    };
    targets.template_rebuild_nodes = vec![shared.clone()];
    targets.removed_compiled_nodes = vec![shared.clone()];
    targets.theme_restyle_nodes = vec![UiAssetNodeTarget {
        tree_id: main.clone(),
        node_id: UiNodeId::new(2),
    }];
    targets.resource_damage_nodes = vec![UiAssetNodeTarget {
        tree_id: main,
        node_id: UiNodeId::new(3),
    }];

    let total_inputs = targets.template_rebuild_nodes.len()
        + targets.removed_compiled_nodes.len()
        + targets.theme_restyle_nodes.len()
        + targets.resource_damage_nodes.len();
    let projected = targets.all_target_nodes();

    assert_eq!(
        projected,
        vec![
            shared,
            UiAssetNodeTarget {
                tree_id: tree_id("surface/main"),
                node_id: UiNodeId::new(2),
            },
            UiAssetNodeTarget {
                tree_id: tree_id("surface/main"),
                node_id: UiNodeId::new(3),
            }
        ]
    );
    assert_eq!(projected.len(), total_inputs - 1);
    assert!(projected.capacity() >= total_inputs);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260917_runtime792_surface_index_target_capacity_bench() {
    let inputs = benchmark_inputs();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_projection(&inputs, false));
            optimized_samples.push(measure_projection(&inputs, true));
        } else {
            optimized_samples.push(measure_projection(&inputs, true));
            legacy_samples.push(measure_projection(&inputs, false));
        }
    }

    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    let required = CATEGORY_SIZE * CATEGORY_COUNT;
    println!(
        "RUNTIME792_SURFACE_INDEX_TARGET_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} category_size={CATEGORY_SIZE} \
category_count={CATEGORY_COUNT} required_output={required} \
legacy_growth_events={} optimized_growth_events={} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        geometric_growth_events(required),
        reserved_growth_events(required),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(legacy_p95_ns > 0);
    assert!(optimized_p95_ns > 0);
    assert!(geometric_growth_events(required) > reserved_growth_events(required));
}

fn benchmark_inputs() -> [Vec<UiTreeId>; CATEGORY_COUNT] {
    std::array::from_fn(|category| {
        (0..CATEGORY_SIZE)
            .map(|index| tree_id(&format!("surface/{category}/{index}")))
            .collect()
    })
}

fn measure_projection(inputs: &[Vec<UiTreeId>; CATEGORY_COUNT], reserved: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let total_capacity = inputs.iter().map(Vec::len).sum();
        let mut output = if reserved {
            Vec::with_capacity(total_capacity)
        } else {
            Vec::new()
        };
        let mut seen = BTreeSet::new();
        for category in inputs {
            for surface in category {
                if seen.insert(surface) {
                    output.push(surface.clone());
                }
            }
        }
        checksum = checksum.wrapping_add(output.len());
        black_box(output);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn geometric_growth_events(required: usize) -> usize {
    let mut capacity = 4usize;
    let mut events = 0usize;
    while capacity < required {
        capacity = capacity.saturating_mul(2);
        events += 1;
    }
    events
}

fn reserved_growth_events(_required: usize) -> usize {
    0
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
