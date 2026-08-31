use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const NODE_COUNT: usize = 2_048;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn node(index: usize) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: format!("projection_node_with_long_identity_{index:05}").into(),
        control_id: format!("ProjectionControlWithLongIdentity{index:05}").into(),
        ..ViewTemplateNodeData::default()
    }
}

fn projection(nodes: impl IntoIterator<Item = ViewTemplateNodeData>) -> ViewTemplateNodeProjection {
    ViewTemplateNodeProjection {
        base_rows: Rc::new(nodes.into_iter().map(Rc::new).collect()),
        row_patches: Rc::new(BTreeMap::new()),
        source_frame: None,
    }
}

fn legacy_source_output_rows(
    source_projection: &ViewTemplateNodeProjection,
    composed_nodes: &[ViewTemplateNodeData],
) -> Vec<Option<usize>> {
    source_projection
        .iter()
        .map(|source| {
            composed_nodes.iter().position(|node| {
                node.node_id == source.node_id && node.control_id == source.control_id
            })
        })
        .collect()
}

#[test]
fn optimization_batch_20260826ah_editor01_hash_index_preserves_source_order_and_first_match() {
    let source_projection = projection([node(7), node(99), node(3)]);
    let mut duplicate = node(7);
    duplicate.text = "second duplicate".into();
    let composed_nodes = vec![node(3), node(7), duplicate];

    assert_eq!(
        source_output_rows(&source_projection, &composed_nodes),
        vec![Some(1), None, Some(0)]
    );
}

#[test]
fn optimization_batch_20260826ah_editor01_projection_composition_uses_first_row_hash_index() {
    let source = include_str!("../projection_composition.rs");
    let mapping = source
        .split("fn source_output_rows")
        .nth(1)
        .and_then(|body| body.split("fn merge_projected_binding_delta").next())
        .expect("source/output row mapping implementation");

    assert!(source.contains("use std::collections::{BTreeMap, HashMap};"));
    assert!(mapping.contains("HashMap::with_capacity(composed_nodes.len())"));
    assert!(mapping.contains(".or_insert(row)"));
    assert!(mapping.contains(".get(&(source.node_id.as_str(), source.control_id.as_str()))"));
    assert!(!mapping.contains(".position("));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ah_editor01_projection_composition_hash_index_performance_evidence() {
    let source_projection = projection((0..NODE_COUNT).rev().map(node));
    let composed_nodes = (0..NODE_COUNT).map(node).collect::<Vec<_>>();
    assert_eq!(
        legacy_source_output_rows(&source_projection, &composed_nodes),
        source_output_rows(&source_projection, &composed_nodes)
    );

    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_source_output_rows(
                black_box(&source_projection),
                black_box(&composed_nodes),
            ));
            linear_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(source_output_rows(
                black_box(&source_projection),
                black_box(&composed_nodes),
            ));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(source_output_rows(
                black_box(&source_projection),
                black_box(&composed_nodes),
            ));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_source_output_rows(
                black_box(&source_projection),
                black_box(&composed_nodes),
            ));
            linear_samples.push(started.elapsed());
        }
    }

    let linear_p95 = percentile_95(&mut linear_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR01_PROJECTION_COMPOSITION_HASH_INDEX_BENCH_V1 \
         nodes={NODE_COUNT} stable_source_order=true first_duplicate_wins=true \
         linear_p95_ns={} hash_p95_ns={}",
        linear_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= linear_p95.as_nanos() * 60,
        "hash-index P95 {:?} exceeded 60% of nested-linear P95 {:?}",
        hash_p95,
        linear_p95,
    );
}
