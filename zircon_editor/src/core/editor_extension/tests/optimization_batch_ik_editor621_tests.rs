use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::editor_authoring_extension::GraphNodeDescriptor;

use super::*;

const NODE_ID_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn node_ids() -> Vec<String> {
    (0..NODE_ID_COUNT)
        .map(|index| format!("graph.generated.palette.node.identity.{index:05}"))
        .collect()
}

fn ordered_unique_count(node_ids: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    node_ids
        .iter()
        .filter(|node_id| unique.insert(node_id.as_str()))
        .count()
}

fn hash_unique_count(node_ids: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(node_ids.len());
    node_ids
        .iter()
        .filter(|node_id| unique.insert(node_id.as_str()))
        .count()
}

fn duplicate_palette() -> GraphNodePaletteDescriptor {
    GraphNodePaletteDescriptor::new(
        "sample.palette",
        AssetTypeId::parse("sample.graph").unwrap(),
    )
    .with_node(GraphNodeDescriptor::new("sample.node", "First", "Sample"))
    .with_node(GraphNodeDescriptor::new("sample.other", "Other", "Sample"))
    .with_node(GraphNodeDescriptor::new(
        "sample.node",
        "Duplicate",
        "Sample",
    ))
}

fn assert_duplicate_node(error: EditorExtensionRegistryError) {
    assert!(matches!(
        error,
        EditorExtensionRegistryError::DuplicateContribution {
            kind: "graph node",
            id,
        } if id == "sample.node"
    ));
}

#[test]
fn optimization_batch_ik_editor621_graph_node_validation_preserves_first_duplicate_error() {
    assert_duplicate_node(validate_graph_node_palette(&duplicate_palette()).unwrap_err());

    let mut registry = EditorExtensionRegistry::default();
    assert_duplicate_node(
        registry
            .register_graph_node_palette(duplicate_palette())
            .unwrap_err(),
    );
}

#[test]
fn optimization_batch_ik_editor621_graph_node_validation_uses_preallocated_hash_membership() {
    let source = include_str!("../../editor_extension.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert_eq!(
        production
            .matches(
                "let mut node_ids = std::collections::HashSet::with_capacity(descriptor.nodes().len());"
            )
            .count(),
        1
    );
    assert_eq!(production.matches("node_ids.insert(node.id())").count(), 1);
    assert!(!production.contains("let mut node_ids = std::collections::BTreeSet::new();"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_ik_editor621_hash_graph_node_validation_performance_evidence() {
    let node_ids = node_ids();
    assert_eq!(
        ordered_unique_count(&node_ids),
        hash_unique_count(&node_ids)
    );

    black_box(ordered_unique_count(black_box(&node_ids)));
    black_box(hash_unique_count(black_box(&node_ids)));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&node_ids)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&node_ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&node_ids)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&node_ids)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR621_HASH_GRAPH_NODE_VALIDATION_BENCH_V1 \
         nodes={NODE_ID_COUNT} borrowed_identity=true \
         ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 40,
        "hash graph-node validation P95 {:?} exceeded 40% of ordered P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
