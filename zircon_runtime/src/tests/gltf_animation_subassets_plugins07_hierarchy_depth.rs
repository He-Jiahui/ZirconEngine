use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const ROOTS_PER_SAMPLE: usize = 8_192;

#[test]
fn gltf_hierarchy_stack_contract_depths() {
    let parents = [None, Some(0), None, Some(2), Some(3)];
    assert_eq!(hierarchy_depths(&parents).unwrap(), [0, 1, 0, 1, 2]);
    assert_eq!(
        hierarchy_depths(&parents).unwrap(),
        legacy_hierarchy_depths(&parents).unwrap()
    );
}

#[test]
#[ignore = "release performance gate"]
fn gltf_hierarchy_stack_performance_release_depths() {
    let parents = vec![None; ROOTS_PER_SAMPLE];
    for _ in 0..4 {
        black_box(measure_legacy(&parents));
        black_box(measure_reused(&parents));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (measure_legacy(&parents), measure_reused(&parents))
        } else {
            let optimized_ns = measure_reused(&parents);
            (measure_legacy(&parents), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }
    report(
        "plugins07_gltf_hierarchy_depth_stack",
        ROOTS_PER_SAMPLE,
        1,
        75,
        &legacy_samples,
        &optimized_samples,
    );
}

fn measure_legacy(parents: &[Option<usize>]) -> u128 {
    let started = Instant::now();
    black_box(legacy_hierarchy_depths(black_box(parents)).unwrap());
    started.elapsed().as_nanos().max(1)
}

fn measure_reused(parents: &[Option<usize>]) -> u128 {
    let started = Instant::now();
    black_box(hierarchy_depths(black_box(parents)).unwrap());
    started.elapsed().as_nanos().max(1)
}

fn legacy_hierarchy_depths(
    parent_by_node: &[Option<usize>],
) -> Result<Vec<usize>, AssetImportError> {
    const UNVISITED: u8 = 0;
    const VISITING: u8 = 1;
    const INDEXED: u8 = 2;

    let mut states = vec![UNVISITED; parent_by_node.len()];
    let mut depths = vec![0; parent_by_node.len()];
    for start in 0..parent_by_node.len() {
        if states[start] == INDEXED {
            continue;
        }
        let mut path = Vec::new();
        let mut current = start;
        let mut depth = loop {
            match states[current] {
                UNVISITED => {
                    states[current] = VISITING;
                    path.push(current);
                    let Some(parent) = parent_by_node[current] else {
                        break 0;
                    };
                    current = parent;
                }
                VISITING => {
                    return Err(AssetImportError::Parse(format!(
                        "gltf node hierarchy contains a cycle at Node{current}"
                    )));
                }
                INDEXED => break depths[current],
                _ => unreachable!(),
            }
        };
        for node_index in path.into_iter().rev() {
            depth += usize::from(parent_by_node[node_index].is_some());
            depths[node_index] = depth;
            states[node_index] = INDEXED;
        }
    }
    Ok(depths)
}

fn report(
    name: &str,
    legacy_stack_allocations_per_sample: usize,
    optimized_stack_allocations_per_sample: usize,
    threshold_percent: u128,
    legacy_samples: &[u128],
    optimized_samples: &[u128],
) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT {name} sample_pairs={SAMPLE_PAIRS} roots_per_sample={ROOTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent} legacy_stack_allocations_per_sample={legacy_stack_allocations_per_sample} optimized_stack_allocations_per_sample={optimized_stack_allocations_per_sample} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        improvement_percent >= threshold_percent,
        "reused glTF hierarchy depth stack missed its P95 improvement gate"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
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
