use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use serde_json::json;
use zircon_runtime_interface::ui::event_ui::{
    UiNodeDescriptor, UiNodeId, UiNodePath, UiPropertyDescriptor, UiReflectionDiff,
    UiReflectionNodePatch, UiReflectionSnapshot, UiTreeId, UiValueType,
};

use super::UiEventManager;

const SAMPLE_PAIRS: usize = 21;

#[test]
fn runtime11a_direct_property_query_preserves_results() {
    let (manager, node_path) = wide_property_manager(8, 64);

    assert_eq!(
        manager
            .query_property(&node_path, "property-4")
            .expect("target property")
            .reflected_value,
        json!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")
    );
    assert!(manager.query_property(&node_path, "missing").is_none());
    assert!(manager
        .query_property(&UiNodePath::new("missing/node"), "property-4")
        .is_none());
}

#[test]
#[ignore = "release performance evidence"]
fn runtime11a_direct_property_query_benchmark_evidence() {
    const PROPERTIES: usize = 256;
    const VALUE_BYTES: usize = 1_024;
    const ITERATIONS: usize = 64;

    let (manager, node_path) = wide_property_manager(PROPERTIES, VALUE_BYTES);
    let mut legacy = || {
        let mut bytes = 0;
        for _ in 0..ITERATIONS {
            let property = manager
                .query_node(black_box(&node_path))
                .and_then(|node| node.properties.get("property-128").cloned())
                .expect("target property");
            bytes += property
                .reflected_value
                .as_str()
                .expect("string property")
                .len();
        }
        bytes
    };
    let mut optimized = || {
        let mut bytes = 0;
        for _ in 0..ITERATIONS {
            let property = manager
                .query_property(black_box(&node_path), "property-128")
                .expect("target property");
            bytes += property
                .reflected_value
                .as_str()
                .expect("string property")
                .len();
        }
        bytes
    };

    let expected = ITERATIONS * VALUE_BYTES;
    assert_eq!(black_box(legacy()), expected);
    assert_eq!(black_box(optimized()), expected);
    let (legacy_ns, optimized_ns) = paired_samples(&mut legacy, &mut optimized, expected);
    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "direct property query P95 must be at least 50% below whole-node cloning: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME11A_DIRECT_PROPERTY_QUERY_BENCH_V1 properties={PROPERTIES} value_bytes={VALUE_BYTES} iterations={ITERATIONS} sample_pairs={SAMPLE_PAIRS} legacy_node_property_clones_per_query={PROPERTIES} optimized_property_clones_per_query=1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

#[test]
#[ignore = "release performance evidence"]
fn runtime11a_borrowed_patch_target_resolution_benchmark_evidence() {
    const PATCHES: usize = 4_096;
    const TREE_ID_BYTES: usize = 16_384;

    let (mut legacy_manager, node_path) = patch_manager(TREE_ID_BYTES);
    let (mut optimized_manager, _) = patch_manager(TREE_ID_BYTES);
    let patches = (0..PATCHES)
        .map(|_| {
            UiReflectionNodePatch::new(node_path.clone())
                .with_property("transient.hovered", json!(false))
        })
        .collect::<Vec<_>>();
    let mut legacy = || legacy_apply_reflection_patches(&mut legacy_manager, black_box(&patches));
    let mut optimized = || {
        optimized_manager
            .apply_reflection_patches(black_box(&patches))
            .expect("valid no-op patches")
            .len()
    };

    assert_eq!(black_box(legacy()), 0);
    assert_eq!(black_box(optimized()), 0);
    let (legacy_ns, optimized_ns) = paired_samples(&mut legacy, &mut optimized, 0);
    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "borrowed patch target resolution P95 must be at least 25% below per-patch tree-id cloning: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME11A_BORROWED_PATCH_TARGET_RESOLUTION_BENCH_V1 patches={PATCHES} tree_id_bytes={TREE_ID_BYTES} sample_pairs={SAMPLE_PAIRS} legacy_tree_id_clones={PATCHES} optimized_tree_id_clones=0 legacy_cloned_tree_id_bytes={} optimized_cloned_tree_id_bytes=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        PATCHES * TREE_ID_BYTES,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn wide_property_manager(
    property_count: usize,
    value_bytes: usize,
) -> (UiEventManager, UiNodePath) {
    let node_path = UiNodePath::new("editor/workbench/inspector");
    let mut node = UiNodeDescriptor::new(
        UiNodeId::new(1),
        node_path.clone(),
        "InspectorView",
        "Inspector",
    );
    let value = "x".repeat(value_bytes);
    for index in 0..property_count {
        node = node.with_property(UiPropertyDescriptor::new(
            format!("property-{index}"),
            UiValueType::String,
            json!(value.clone()),
        ));
    }
    let mut manager = UiEventManager::default();
    manager.replace_tree(UiReflectionSnapshot::new(
        UiTreeId::new("editor.workbench"),
        vec![UiNodeId::new(1)],
        vec![node],
    ));
    (manager, node_path)
}

fn patch_manager(tree_id_bytes: usize) -> (UiEventManager, UiNodePath) {
    let node_path = UiNodePath::new("editor/workbench/scene");
    let node = UiNodeDescriptor::new(UiNodeId::new(1), node_path.clone(), "SceneView", "Scene")
        .with_property(UiPropertyDescriptor::new(
            "transient.hovered",
            UiValueType::Bool,
            json!(false),
        ));
    let mut manager = UiEventManager::default();
    manager.replace_tree(UiReflectionSnapshot::new(
        UiTreeId::new("t".repeat(tree_id_bytes)),
        vec![UiNodeId::new(1)],
        vec![node],
    ));
    (manager, node_path)
}

fn legacy_apply_reflection_patches(
    manager: &mut UiEventManager,
    patches: &[UiReflectionNodePatch],
) -> usize {
    let mut resolved = Vec::with_capacity(patches.len());
    for (patch_index, patch) in patches.iter().enumerate() {
        let (tree_id, node_id) = manager
            .node_index
            .get(&patch.node_path)
            .cloned()
            .expect("known node");
        let node = manager
            .trees
            .get(&tree_id)
            .and_then(|tree| tree.nodes.get(&node_id))
            .expect("known indexed node");
        assert!(patch
            .properties
            .keys()
            .all(|property_name| node.properties.contains_key(property_name)));
        resolved.push((tree_id, node_id, patch_index));
    }

    let mut changed_by_tree = BTreeMap::<UiTreeId, BTreeSet<_>>::new();
    for (tree_id, node_id, patch_index) in resolved {
        let patch = &patches[patch_index];
        let node = manager
            .trees
            .get_mut(&tree_id)
            .and_then(|tree| tree.nodes.get_mut(&node_id))
            .expect("validated patch target");
        let mut changed = false;
        for (property_name, value) in &patch.properties {
            let property = node
                .properties
                .get_mut(property_name)
                .expect("known property");
            if property.reflected_value != *value {
                property.reflected_value = value.clone();
                changed = true;
            }
        }
        if changed {
            changed_by_tree.entry(tree_id).or_default().insert(node_id);
        }
    }

    changed_by_tree
        .into_iter()
        .map(|(tree_id, changed_nodes)| UiReflectionDiff {
            tree_id,
            changed_nodes: changed_nodes.into_iter().collect(),
            removed_nodes: Vec::new(),
        })
        .count()
}

fn paired_samples(
    legacy: &mut impl FnMut() -> usize,
    optimized: &mut impl FnMut() -> usize,
    expected: usize,
) -> (Vec<u128>, Vec<u128>) {
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(legacy, expected));
            optimized_ns.push(measure_ns(optimized, expected));
        } else {
            optimized_ns.push(measure_ns(optimized, expected));
            legacy_ns.push(measure_ns(legacy, expected));
        }
    }
    (legacy_ns, optimized_ns)
}

fn measure_ns(operation: &mut impl FnMut() -> usize, expected: usize) -> u128 {
    let started = Instant::now();
    assert_eq!(black_box(operation()), expected);
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
