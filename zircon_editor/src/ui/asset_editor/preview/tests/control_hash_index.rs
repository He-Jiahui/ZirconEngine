use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetHeader, UiAssetKind, UiChildMount, UiNodeDefinition,
};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826aq_preview_control_hash_index_preserves_first_duplicate() {
    let document = document_with_controls(4, true);
    let expected_duplicate = document
        .iter_nodes()
        .find(|node| node.control_id.as_deref() == Some("control-00000"))
        .expect("duplicate control");

    let index = control_id_index(&document);

    assert_eq!(index.len(), 4);
    assert!(std::ptr::eq(
        *index.get("control-00000").expect("indexed duplicate"),
        expected_duplicate
    ));
    assert_eq!(
        index.get("control-00003").map(|node| node.node_id.as_str()),
        Some("node-00003")
    );
}

#[test]
fn optimization_batch_20260826aq_preview_control_index_uses_linear_hash_build() {
    let source = include_str!("../preview_projection.rs");
    let index = source
        .split("fn control_id_index")
        .nth(1)
        .expect("control index function")
        .split("mod control_hash_index_tests")
        .next()
        .expect("bounded control index");

    assert!(source.contains("use std::collections::HashMap"));
    assert!(index.contains("-> HashMap<&str, &UiNodeDefinition>"));
    assert!(index.contains("HashMap::new()"));
    assert!(index.contains("index.entry(control_id).or_insert(node)"));
    assert!(!index.contains("BTreeMap"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826aq_preview_control_hash_index_p95() {
    const NODES: usize = 16_384;
    let document = document_with_controls(NODES, false);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(|| legacy_control_id_index(&document).len()));
            optimized_ns.push(measure_ns(|| control_id_index(&document).len()));
        } else {
            optimized_ns.push(measure_ns(|| control_id_index(&document).len()));
            legacy_ns.push(measure_ns(|| legacy_control_id_index(&document).len()));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "preview control hash-index P95 must be at least 25% below BTree construction: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_PREVIEW_CONTROL_HASH_INDEX_BENCH_V1 nodes={NODES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_index_complexity=n_log_n optimized_index_complexity=n legacy_tree_insertions_per_sample={NODES} optimized_hash_insertions_per_sample={NODES} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn document_with_controls(nodes: usize, duplicate_first: bool) -> UiAssetDocument {
    let children = (0..nodes)
        .map(|index| UiChildMount {
            node: UiNodeDefinition {
                node_id: format!("node-{index:05}"),
                control_id: Some(format!("control-{index:05}")),
                ..UiNodeDefinition::default()
            },
            ..UiChildMount::default()
        })
        .collect();
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Layout,
            id: "ui.preview.control-index".to_string(),
            version: 1,
            display_name: "Control Index".to_string(),
        },
        imports: Default::default(),
        tokens: Default::default(),
        root: Some(UiNodeDefinition {
            node_id: "root".to_string(),
            control_id: duplicate_first.then(|| "control-00000".to_string()),
            children,
            ..UiNodeDefinition::default()
        }),
        components: Default::default(),
        stylesheets: Vec::new(),
    }
}

fn legacy_control_id_index(document: &UiAssetDocument) -> BTreeMap<&str, &UiNodeDefinition> {
    let mut index = BTreeMap::new();
    for node in document.iter_nodes() {
        if let Some(control_id) = node.control_id.as_deref() {
            let _ = index.entry(control_id).or_insert(node);
        }
    }
    index
}

fn measure_ns(operation: impl FnOnce() -> usize) -> u128 {
    let started = Instant::now();
    assert!(black_box(operation()) > 0);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
