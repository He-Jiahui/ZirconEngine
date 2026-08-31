use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{
    materialize_prototype_nodes, prototype_node_handles, validate_reachable_prototype_root,
    FlatUiChildMount, FlatUiNodeDefinition,
};
use zircon_runtime_interface::ui::template::{
    UiAssetError, UiNodePrototype, UiPrototypeChildMount, UiPrototypeNodeHandle,
};

const BENCHMARK_NODE_COUNT: usize = 10_000;
const BENCHMARK_WARMUP_PAIRS: usize = 4;
const BENCHMARK_SAMPLE_PAIRS: usize = 21;
const LEGACY_NODE_CLONES: usize = 17;
const LEGACY_EDGE_CLONES: usize = 2;

#[test]
fn flat_prototype_owned_materialization_matches_legacy_projection() {
    let flat_nodes = fixture_nodes(64);
    let handles = prototype_node_handles("runtime73-fixture", &flat_nodes).unwrap();
    let legacy =
        legacy_materialize_prototype_nodes("runtime73-fixture", &flat_nodes, &handles).unwrap();
    let optimized = materialize_prototype_nodes("runtime73-fixture", flat_nodes, &handles).unwrap();

    assert_eq!(optimized, legacy);
}

#[test]
fn flat_prototype_borrowed_reachability_validation_matches_owned_traversal() {
    let nodes = fixture_nodes(64);
    let handles = prototype_node_handles("runtime73-fixture", &nodes).unwrap();
    assert_eq!(
        validate_reachable_prototype_root(
            "runtime73-fixture",
            &nodes,
            &handles,
            Some("node-00000"),
        ),
        legacy_validate_reachable_prototype_root(
            "runtime73-fixture",
            &nodes,
            &handles,
            Some("node-00000"),
        )
    );

    let mut missing = nodes.clone();
    missing
        .get_mut("node-00017")
        .unwrap()
        .children
        .push(FlatUiChildMount {
            child: "missing-node".to_string(),
            ..FlatUiChildMount::default()
        });
    assert_eq!(
        validate_reachable_prototype_root(
            "runtime73-fixture",
            &missing,
            &handles,
            Some("node-00000"),
        ),
        legacy_validate_reachable_prototype_root(
            "runtime73-fixture",
            &missing,
            &handles,
            Some("node-00000"),
        )
    );

    let mut cyclic = nodes;
    cyclic
        .get_mut("node-00063")
        .unwrap()
        .children
        .push(FlatUiChildMount {
            child: "node-00000".to_string(),
            ..FlatUiChildMount::default()
        });
    assert_eq!(
        validate_reachable_prototype_root(
            "runtime73-fixture",
            &cyclic,
            &handles,
            Some("node-00000"),
        ),
        legacy_validate_reachable_prototype_root(
            "runtime73-fixture",
            &cyclic,
            &handles,
            Some("node-00000"),
        )
    );
}

#[test]
#[ignore = "performance acceptance benchmark"]
fn flat_prototype_owned_field_move_performance_acceptance() {
    let template = fixture_nodes(BENCHMARK_NODE_COUNT);
    let handles = prototype_node_handles("runtime73-benchmark", &template).unwrap();

    for _ in 0..BENCHMARK_WARMUP_PAIRS {
        let warm_legacy = template.clone();
        black_box(
            legacy_materialize_prototype_nodes("runtime73-benchmark", &warm_legacy, &handles)
                .unwrap(),
        );
        black_box(
            materialize_prototype_nodes("runtime73-benchmark", template.clone(), &handles).unwrap(),
        );
    }

    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    for pair in 0..BENCHMARK_SAMPLE_PAIRS {
        let legacy_input = template.clone();
        let optimized_input = template.clone();
        if pair % 2 == 0 {
            legacy_samples.push(time_legacy(&legacy_input, &handles));
            optimized_samples.push(time_optimized(optimized_input, &handles));
        } else {
            optimized_samples.push(time_optimized(optimized_input, &handles));
            legacy_samples.push(time_legacy(&legacy_input, &handles));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&optimized_samples, 50);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    let edge_count = BENCHMARK_NODE_COUNT - 1;
    let legacy_owned_clones =
        BENCHMARK_NODE_COUNT * LEGACY_NODE_CLONES + edge_count * LEGACY_EDGE_CLONES;
    let optimized_owned_clones = BENCHMARK_NODE_COUNT;

    println!(
        "RUNTIME73_FLAT_PROTOTYPE_MOVE_PERF nodes={} edges={} warmup_pairs={} pairs={} order=alternating percentile=nearest-rank legacy_owned_clones={} optimized_owned_clones={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        BENCHMARK_NODE_COUNT,
        edge_count,
        BENCHMARK_WARMUP_PAIRS,
        BENCHMARK_SAMPLE_PAIRS,
        legacy_owned_clones,
        optimized_owned_clones,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );

    assert_eq!(legacy_owned_clones, 189_998);
    assert_eq!(optimized_owned_clones, 10_000);
    assert!(
        optimized_p95_ns * 2 <= legacy_p95_ns,
        "owned-field move must reduce projection P95 by at least 50%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "performance acceptance benchmark"]
fn flat_prototype_borrowed_validation_performance_acceptance() {
    let nodes = fixture_nodes(BENCHMARK_NODE_COUNT);
    let handles = prototype_node_handles("runtime73-benchmark", &nodes).unwrap();

    for _ in 0..BENCHMARK_WARMUP_PAIRS {
        black_box(
            legacy_validate_reachable_prototype_root(
                "runtime73-benchmark",
                &nodes,
                &handles,
                Some("node-00000"),
            )
            .unwrap(),
        );
        black_box(
            validate_reachable_prototype_root(
                "runtime73-benchmark",
                &nodes,
                &handles,
                Some("node-00000"),
            )
            .unwrap(),
        );
    }

    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    for pair in 0..BENCHMARK_SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(time_legacy_validation(&nodes, &handles));
            optimized_samples.push(time_borrowed_validation(&nodes, &handles));
        } else {
            optimized_samples.push(time_borrowed_validation(&nodes, &handles));
            legacy_samples.push(time_legacy_validation(&nodes, &handles));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&optimized_samples, 50);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);

    println!(
        "RUNTIME73_FLAT_PROTOTYPE_VALIDATION_PERF nodes={} edges={} warmup_pairs={} pairs={} order=alternating percentile=nearest-rank legacy_node_id_clones={} optimized_node_id_clones=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        BENCHMARK_NODE_COUNT,
        BENCHMARK_NODE_COUNT - 1,
        BENCHMARK_WARMUP_PAIRS,
        BENCHMARK_SAMPLE_PAIRS,
        BENCHMARK_NODE_COUNT * 2,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );

    assert_eq!(BENCHMARK_NODE_COUNT * 2, 20_000);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(60),
        "borrowed reachability validation must reduce P95 by at least 40%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn time_legacy(
    flat_nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    handles: &BTreeMap<String, UiPrototypeNodeHandle>,
) -> u128 {
    let started = Instant::now();
    let output =
        legacy_materialize_prototype_nodes("runtime73-benchmark", flat_nodes, handles).unwrap();
    let elapsed = started.elapsed().as_nanos();
    black_box(output.len());
    elapsed
}

fn time_optimized(
    flat_nodes: BTreeMap<String, FlatUiNodeDefinition>,
    handles: &BTreeMap<String, UiPrototypeNodeHandle>,
) -> u128 {
    let started = Instant::now();
    let output = materialize_prototype_nodes("runtime73-benchmark", flat_nodes, handles).unwrap();
    let elapsed = started.elapsed().as_nanos();
    black_box(output.len());
    elapsed
}

fn time_legacy_validation(
    nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    handles: &BTreeMap<String, UiPrototypeNodeHandle>,
) -> u128 {
    let started = Instant::now();
    let result = legacy_validate_reachable_prototype_root(
        "runtime73-benchmark",
        nodes,
        handles,
        Some("node-00000"),
    );
    let elapsed = started.elapsed().as_nanos();
    black_box(result.unwrap());
    elapsed
}

fn time_borrowed_validation(
    nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    handles: &BTreeMap<String, UiPrototypeNodeHandle>,
) -> u128 {
    let started = Instant::now();
    let result = validate_reachable_prototype_root(
        "runtime73-benchmark",
        nodes,
        handles,
        Some("node-00000"),
    );
    let elapsed = started.elapsed().as_nanos();
    black_box(result.unwrap());
    elapsed
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn fixture_nodes(count: usize) -> BTreeMap<String, FlatUiNodeDefinition> {
    (0..count)
        .map(|index| {
            let node_id = fixture_node_id(index);
            let children = (index + 1 < count)
                .then(|| FlatUiChildMount {
                    child: fixture_node_id(index + 1),
                    mount: Some("content".to_string()),
                    slot: BTreeMap::from([("gap".to_string(), Value::Integer(index as i64))]),
                })
                .into_iter()
                .collect();
            let node = FlatUiNodeDefinition {
                widget_type: Some("Stack".to_string()),
                component: Some("Runtime73BenchmarkComponent".to_string()),
                component_ref: Some("runtime73.benchmark.component".to_string()),
                slot_name: Some("content".to_string()),
                control_id: Some(format!("control-{index:05}")),
                classes: vec!["benchmark".to_string(), "selected".to_string()],
                params: BTreeMap::from([(
                    "label".to_string(),
                    Value::String(format!("node-{index:05}-label")),
                )]),
                props: BTreeMap::from([("enabled".to_string(), Value::Boolean(true))]),
                layout: Some(BTreeMap::from([("width".to_string(), Value::Integer(320))])),
                children,
                ..FlatUiNodeDefinition::default()
            };
            (node_id, node)
        })
        .collect()
}

fn fixture_node_id(index: usize) -> String {
    format!("node-{index:05}")
}

fn legacy_materialize_prototype_nodes(
    asset_id: &str,
    flat_nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    node_handles: &BTreeMap<String, UiPrototypeNodeHandle>,
) -> Result<Vec<UiNodePrototype>, UiAssetError> {
    let mut nodes = vec![UiNodePrototype::default(); node_handles.len()];
    for (node_id, flat_node) in flat_nodes {
        let handle = node_handles[node_id];
        nodes[handle.index()] = legacy_clone_node(flat_node, asset_id, node_id, node_handles)?;
    }
    Ok(nodes)
}

fn legacy_clone_node(
    node: &FlatUiNodeDefinition,
    asset_id: &str,
    node_id: &str,
    node_handles: &BTreeMap<String, UiPrototypeNodeHandle>,
) -> Result<UiNodePrototype, UiAssetError> {
    let children = node
        .children
        .iter()
        .map(|child| {
            let child_handle = node_handles.get(&child.child).copied().ok_or_else(|| {
                UiAssetError::MissingNode {
                    asset_id: asset_id.to_string(),
                    node_id: child.child.clone(),
                }
            })?;
            Ok(UiPrototypeChildMount {
                mount: child.mount.clone(),
                slot: child.slot.clone(),
                child: child_handle,
            })
        })
        .collect::<Result<_, UiAssetError>>()?;

    Ok(UiNodePrototype {
        node_id: node_id.to_string(),
        kind: node.kind,
        widget_type: node.widget_type.clone(),
        component: node.component.clone(),
        component_ref: node.component_ref.clone(),
        slot_name: node.slot_name.clone(),
        control_id: node.control_id.clone(),
        classes: node.classes.clone(),
        params: node.params.clone(),
        props: node.props.clone(),
        layout: node.layout.clone(),
        bindings: node.bindings.clone(),
        style_overrides: node.style_overrides.clone(),
        focus: node.focus.clone(),
        navigation: node.navigation.clone(),
        picking: node.picking,
        a11y: node.a11y.clone(),
        widget: node.widget.clone(),
        children,
    })
}

fn legacy_validate_reachable_prototype_root(
    asset_id: &str,
    nodes: &BTreeMap<String, FlatUiNodeDefinition>,
    node_handles: &BTreeMap<String, UiPrototypeNodeHandle>,
    root: Option<&str>,
) -> Result<(), UiAssetError> {
    let Some(root) = root else {
        return Ok(());
    };
    if !node_handles.contains_key(root) {
        return Err(UiAssetError::MissingNode {
            asset_id: asset_id.to_string(),
            node_id: root.to_string(),
        });
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = vec![LegacyPrototypeVisitFrame::Enter(root.to_string())];
    while let Some(frame) = stack.pop() {
        match frame {
            LegacyPrototypeVisitFrame::Enter(node_id) => {
                if visited.contains(&node_id) {
                    continue;
                }
                if !visiting.insert(node_id.clone()) {
                    return Err(UiAssetError::InvalidDocument {
                        asset_id: asset_id.to_string(),
                        detail: format!("ui asset prototype contains a cycle at {node_id}"),
                    });
                }
                let node = nodes
                    .get(&node_id)
                    .ok_or_else(|| UiAssetError::MissingNode {
                        asset_id: asset_id.to_string(),
                        node_id: node_id.clone(),
                    })?;
                stack.push(LegacyPrototypeVisitFrame::Exit(node_id));
                for child in node.children.iter().rev() {
                    if !node_handles.contains_key(&child.child) {
                        return Err(UiAssetError::MissingNode {
                            asset_id: asset_id.to_string(),
                            node_id: child.child.clone(),
                        });
                    }
                    stack.push(LegacyPrototypeVisitFrame::Enter(child.child.clone()));
                }
            }
            LegacyPrototypeVisitFrame::Exit(node_id) => {
                let _ = visiting.remove(&node_id);
                let _ = visited.insert(node_id);
            }
        }
    }
    Ok(())
}

enum LegacyPrototypeVisitFrame {
    Enter(String),
    Exit(String),
}
