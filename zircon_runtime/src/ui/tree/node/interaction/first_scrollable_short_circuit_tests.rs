use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiContainerKind, UiScrollableBoxConfig},
    tree::{UiTree, UiTreeError, UiTreeNode},
};

use super::UiRuntimeTreeInteractionExt;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 8_192;
const CANDIDATE_COUNT: usize = 256;

#[test]
fn optimization_batch_20260826eg_runtime176_first_scrollable_preserves_order_and_errors() {
    let mut tree = UiTree::new(UiTreeId::new("first-scrollable"));
    tree.insert_root(UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("plain")));
    tree.insert_root(scrollable_node(2));
    tree.insert_root(scrollable_node(3));

    assert_eq!(
        tree.first_scrollable_in_candidates(&[
            UiNodeId::new(1),
            UiNodeId::new(2),
            UiNodeId::new(3),
        ]),
        Ok(Some(UiNodeId::new(2)))
    );
    assert!(matches!(
        tree.first_scrollable_in_candidates(&[UiNodeId::new(99), UiNodeId::new(2)]),
        Err(UiTreeError::MissingNode(UiNodeId(99)))
    ));
    assert_eq!(
        tree.first_scrollable_in_candidates(&[UiNodeId::new(2), UiNodeId::new(99)]),
        Ok(Some(UiNodeId::new(2)))
    );
}

#[test]
fn optimization_batch_20260826eg_runtime176_first_scrollable_avoids_full_collection() {
    let source = include_str!("../interaction.rs");
    let impl_start = source
        .find("impl UiRuntimeTreeInteractionExt for UiTree")
        .unwrap();
    let function_start = source[impl_start..]
        .find("fn first_scrollable_in_candidates")
        .map(|offset| impl_start + offset)
        .unwrap();
    let function_end = source[function_start..]
        .find("fn scrollable_candidates")
        .map(|offset| function_start + offset)
        .unwrap();
    let function_source = &source[function_start..function_end];
    assert!(!function_source.contains("scrollable_candidates(candidates)?"));
    assert!(!function_source.contains("Vec<"));
    assert!(function_source.contains("return Ok(Some(*node_id))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eg_runtime176_first_scrollable_short_circuit_bench() {
    let (tree, candidates) = full_scrollable_fixture();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&tree, &candidates));
            optimized_samples.push(measure_optimized(&tree, &candidates));
        } else {
            optimized_samples.push(measure_optimized(&tree, &candidates));
            legacy_samples.push(measure_legacy(&tree, &candidates));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME176_FIRST_SCROLLABLE_SHORT_CIRCUIT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} candidates_per_lookup={CANDIDATE_COUNT} \
legacy_candidate_visits_per_first_hit={CANDIDATE_COUNT} optimized_candidate_visits_per_first_hit=1 \
legacy_output_allocations_per_lookup=1 optimized_output_allocations_per_lookup=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "short-circuit first scrollable P95 {optimized_p95_ns}ns must be at most 70% of full collection P95 {legacy_p95_ns}ns"
    );
}

fn scrollable_node(id: u64) -> UiTreeNode {
    UiTreeNode::new(
        UiNodeId::new(id),
        UiNodePath::new(format!("scrollable-{id}")),
    )
    .with_container(UiContainerKind::ScrollableBox(
        UiScrollableBoxConfig::default(),
    ))
}

fn full_scrollable_fixture() -> (UiTree, Vec<UiNodeId>) {
    let mut tree = UiTree::new(UiTreeId::new("first-scrollable-benchmark"));
    let candidates = (1..=CANDIDATE_COUNT as u64)
        .map(UiNodeId::new)
        .collect::<Vec<_>>();
    for node_id in &candidates {
        tree.insert_root(scrollable_node(node_id.0));
    }
    (tree, candidates)
}

fn legacy_first_scrollable(
    tree: &UiTree,
    candidates: &[UiNodeId],
) -> Result<Option<UiNodeId>, UiTreeError> {
    Ok(tree.scrollable_candidates(candidates)?.into_iter().next())
}

fn measure_legacy(tree: &UiTree, candidates: &[UiNodeId]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(legacy_first_scrollable(
            black_box(tree),
            black_box(candidates),
        ))
        .unwrap()
        .unwrap()
        .0;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(tree: &UiTree, candidates: &[UiNodeId]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^=
            black_box(black_box(tree).first_scrollable_in_candidates(black_box(candidates)))
                .unwrap()
                .unwrap()
                .0;
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
