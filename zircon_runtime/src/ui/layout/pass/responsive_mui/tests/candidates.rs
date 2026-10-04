use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath},
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
};

use super::MuiResponsiveCandidates;

fn media_query_node(query: &str) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root/query")).with_template_metadata(
        UiTemplateNodeMetadata {
            component: "UseMediaQuery".to_string(),
            attributes: [("query".to_string(), Value::String(query.to_string()))]
                .into_iter()
                .collect(),
            ..UiTemplateNodeMetadata::default()
        },
    )
}

#[test]
fn responsive_width_gate_reuses_candidates_inside_one_threshold_band() {
    let mut tree = UiTree::default();
    tree.nodes
        .insert(UiNodeId::new(1), media_query_node("(min-width: 600px)"));
    let mut candidates = MuiResponsiveCandidates::for_tree(&tree);

    assert!(candidates.responsive_layout_may_change(300.0));
    assert!(!candidates.responsive_layout_may_change(599.0));
    assert!(candidates.responsive_layout_may_change(600.0));
    assert!(!candidates.responsive_layout_may_change(899.0));
    assert!(candidates.responsive_layout_may_change(900.0));
}

#[test]
fn responsive_candidate_mutation_reopens_gate_at_the_same_width() {
    let mut tree = UiTree::default();
    tree.nodes
        .insert(UiNodeId::new(1), media_query_node("(min-width: 600px)"));
    let mut candidates = MuiResponsiveCandidates::for_tree(&tree);
    assert!(candidates.responsive_layout_may_change(800.0));
    assert!(!candidates.responsive_layout_may_change(800.0));

    tree.nodes
        .insert(UiNodeId::new(1), media_query_node("(min-width: 900px)"));
    candidates.patch_nodes(&tree, &[UiNodeId::new(1)].into_iter().collect());

    assert!(candidates.responsive_layout_may_change(800.0));
}

#[test]
fn responsive_grid_item_mutation_reopens_gate_at_the_same_width() {
    let node_id = UiNodeId::new(1);
    let mut tree = UiTree::default();
    tree.nodes.insert(
        node_id,
        UiTreeNode::new(node_id, UiNodePath::new("root/grid-item")).with_template_metadata(
            UiTemplateNodeMetadata {
                component: "GridItem".to_string(),
                attributes: [("size".to_string(), Value::Integer(6))]
                    .into_iter()
                    .collect(),
                ..UiTemplateNodeMetadata::default()
            },
        ),
    );
    let mut candidates = MuiResponsiveCandidates::for_tree(&tree);
    assert!(candidates.responsive_layout_may_change(800.0));
    assert!(!candidates.responsive_layout_may_change(800.0));

    tree.nodes.insert(
        node_id,
        UiTreeNode::new(node_id, UiNodePath::new("root/grid-item")).with_template_metadata(
            UiTemplateNodeMetadata {
                component: "GridItem".to_string(),
                attributes: [("size".to_string(), Value::Integer(8))]
                    .into_iter()
                    .collect(),
                ..UiTemplateNodeMetadata::default()
            },
        ),
    );
    candidates.patch_nodes(&tree, &[node_id].into_iter().collect());

    assert!(candidates.responsive_layout_may_change(800.0));
}

#[test]
fn responsive_max_width_crossing_reopens_after_strict_boundary() {
    let mut tree = UiTree::default();
    tree.nodes
        .insert(UiNodeId::new(1), media_query_node("(max-width: 800px)"));
    let mut candidates = MuiResponsiveCandidates::for_tree(&tree);

    assert!(candidates.responsive_layout_may_change(800.0));
    assert!(candidates.responsive_layout_may_change(801.0));
}

#[test]
fn optimization_batch_gy_runtime580_definition_is_moved_into_the_index() {
    let source = include_str!("../candidates.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("responsive candidate production source");

    assert!(production.contains("if let Some(definition) = next_definition"));
    assert!(production.contains("insert(node_id, definition)"));
    assert!(!production.contains("insert(node_id, definition.clone())"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_gy_runtime580_definition_move_performance_evidence() {
    fn definition(index: usize) -> super::ResponsiveDefinition {
        super::ResponsiveDefinition {
            component: format!("ResponsivePanel{index}"),
            attributes: (0..24)
                .map(|attribute| {
                    (
                        format!("attribute-{attribute}"),
                        Value::String(format!("value-{index}-{attribute}")),
                    )
                })
                .collect(),
        }
    }

    const DEFINITION_COUNT: usize = 4_096;
    let definitions = (0..DEFINITION_COUNT).map(definition).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let legacy_input = definitions.clone();
        let started = Instant::now();
        let mut legacy = BTreeMap::new();
        for (index, definition) in legacy_input.iter().enumerate() {
            legacy.insert(UiNodeId::new(index as u64), definition.clone());
        }
        black_box(legacy);
        legacy_samples.push(started.elapsed().as_nanos());

        let optimized_input = definitions.clone();
        let started = Instant::now();
        let mut optimized = BTreeMap::new();
        for (index, definition) in optimized_input.into_iter().enumerate() {
            optimized.insert(UiNodeId::new(index as u64), definition);
        }
        black_box(optimized);
        optimized_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "RUNTIME580_RESPONSIVE_DEFINITION_MOVE_BENCH_V1 definitions={DEFINITION_COUNT} attributes_per_definition=24 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} target_ratio_bp=7000"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "responsive definition move P95 {optimized_p95} ns exceeded 70% of legacy {legacy_p95} ns"
    );
}
