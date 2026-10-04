use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use zircon_runtime_interface::ui::tree::{UiTemplateNodeMetadata, UiTree, UiTreeError, UiTreeNode};

use super::{runtime_selector_path, SelectorPathNode};

#[test]
fn runtime854_runtime_selector_path_preserves_order_and_host() {
    let (tree, target_id) = tree_with_chain(3);
    let component_states = crate::ui::surface::UiSurfaceComponentStateStore::default();
    let path = runtime_selector_path(&tree, &component_states, target_id)
        .expect("all nodes in the selector path are present");

    assert_eq!(
        path.iter()
            .map(|node| node.component.as_str())
            .collect::<Vec<_>>(),
        vec!["root", "middle", "leaf"]
    );
    assert!(path.first().is_some_and(|node| node.is_host));
    assert!(path.iter().skip(1).all(|node| !node.is_host));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime854_runtime_selector_path_single_buffer_release_benchmark() {
    const DEPTH: usize = 128;
    const BUILDS_PER_SAMPLE: usize = 1_024;
    const SAMPLE_PAIRS: usize = 17;
    let (tree, target_id) = tree_with_chain(DEPTH);
    let component_states = crate::ui::surface::UiSurfaceComponentStateStore::default();
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_path(
                &tree,
                &component_states,
                target_id,
                BUILDS_PER_SAMPLE,
                false,
            ));
            optimized_ns.push(measure_path(
                &tree,
                &component_states,
                target_id,
                BUILDS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_ns.push(measure_path(
                &tree,
                &component_states,
                target_id,
                BUILDS_PER_SAMPLE,
                true,
            ));
            legacy_ns.push(measure_path(
                &tree,
                &component_states,
                target_id,
                BUILDS_PER_SAMPLE,
                false,
            ));
        }
    }
    println!(
        "RUNTIME854_RUNTIME_SELECTOR_PATH_SINGLE_BUFFER_BENCH_V1 depth={DEPTH} builds_per_sample={BUILDS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_intermediate_buffers_per_build=1 optimized_intermediate_buffers_per_build=0 legacy_p50_ns={} legacy_p95_ns={} legacy_p99_ns={} optimized_p50_ns={} optimized_p95_ns={} optimized_p99_ns={} legacy_raw_ns={} optimized_raw_ns={}",
        percentile(&legacy_ns, 50),
        percentile(&legacy_ns, 95),
        percentile(&legacy_ns, 99),
        percentile(&optimized_ns, 50),
        percentile(&optimized_ns, 95),
        percentile(&optimized_ns, 99),
        csv(&legacy_ns),
        csv(&optimized_ns),
    );
}

fn tree_with_chain(depth: usize) -> (UiTree, UiNodeId) {
    assert!(depth > 0);
    let mut tree = UiTree::new(UiTreeId::new("runtime854-selector-path"));
    let root_id = UiNodeId::new(1);
    tree.insert_root(node(root_id, "root"));
    let mut parent_id = root_id;
    for index in 1..depth {
        let child_id = UiNodeId::new((index + 1) as u64);
        let component = if index + 1 == depth { "leaf" } else { "middle" };
        tree.insert_child(parent_id, node(child_id, component))
            .expect("chain parent exists");
        parent_id = child_id;
    }
    (tree, parent_id)
}

fn node(node_id: UiNodeId, component: &str) -> UiTreeNode {
    UiTreeNode::new(node_id, UiNodePath::new(component)).with_template_metadata(
        UiTemplateNodeMetadata {
            component: component.to_owned(),
            ..Default::default()
        },
    )
}

fn legacy_selector_path(
    tree: &UiTree,
    component_states: &crate::ui::surface::UiSurfaceComponentStateStore,
    node_id: UiNodeId,
) -> Result<Vec<SelectorPathNode>, UiTreeError> {
    let mut ids = Vec::new();
    let mut current = Some(node_id);
    while let Some(current_id) = current {
        let node = tree
            .nodes
            .get(&current_id)
            .ok_or(UiTreeError::MissingNode(current_id))?;
        ids.push(current_id);
        current = node.parent;
    }
    ids.reverse();
    let mut path = Vec::with_capacity(ids.len());
    for (index, id) in ids.into_iter().enumerate() {
        let node = tree.nodes.get(&id).ok_or(UiTreeError::MissingNode(id))?;
        path.push(SelectorPathNode::from_tree_node(
            node,
            component_states.get(id),
            index == 0,
        ));
    }
    Ok(path)
}

fn measure_path(
    tree: &UiTree,
    component_states: &crate::ui::surface::UiSurfaceComponentStateStore,
    node_id: UiNodeId,
    builds: usize,
    optimized: bool,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..builds {
        let path = (if optimized {
            runtime_selector_path(tree, component_states, node_id)
        } else {
            legacy_selector_path(tree, component_states, node_id)
        })
        .expect("benchmark tree remains valid");
        checksum = checksum.wrapping_add(path.len());
        black_box(path);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
