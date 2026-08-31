use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiAssetImports, UiAssetKind, UiChildMount, UiComponentDefinition,
    UiNodeDefinition,
};

use super::*;

const NODE_COUNT: usize = 2_048;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn node(node_id: impl Into<String>, children: Vec<UiNodeDefinition>) -> UiNodeDefinition {
    UiNodeDefinition {
        node_id: node_id.into(),
        children: children
            .into_iter()
            .map(|node| UiChildMount {
                node,
                ..UiChildMount::default()
            })
            .collect(),
        ..UiNodeDefinition::default()
    }
}

fn document(
    root: Option<UiNodeDefinition>,
    components: BTreeMap<String, UiComponentDefinition>,
) -> UiAssetDocument {
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Layout,
            id: "editor.hierarchy.streaming_fixture".to_string(),
            version: 1,
            display_name: String::new(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root,
        components,
        stylesheets: Vec::new(),
    }
}

fn legacy_hierarchy_node_ids(document: &UiAssetDocument) -> Vec<String> {
    fn visit(output: &mut Vec<String>, document: &UiAssetDocument, node_id: &str) {
        output.push(node_id.to_string());
        let Some(node) = document.node(node_id) else {
            return;
        };
        for child in &node.children {
            visit(output, document, &child.node.node_id);
        }
    }

    let mut items = Vec::new();
    if let Some(root_id) = document.root_node_id() {
        visit(&mut items, document, root_id);
    } else {
        for component in document.components.values() {
            visit(&mut items, document, &component.root.node_id);
        }
    }
    items
}

fn legacy_selected_hierarchy_index(document: &UiAssetDocument, target: &str) -> i32 {
    legacy_hierarchy_node_ids(document)
        .iter()
        .position(|node_id| node_id == target)
        .map(|index| index as i32)
        .unwrap_or(-1)
}

#[test]
fn optimization_batch_20260826ai_editor23_streaming_hierarchy_preserves_root_and_component_order() {
    let root_document = document(
        Some(node(
            "root",
            vec![
                node("first", vec![node("grandchild", Vec::new())]),
                node("last", Vec::new()),
            ],
        )),
        BTreeMap::from([(
            "ignored_component".to_string(),
            UiComponentDefinition {
                root: node("ignored", Vec::new()),
                ..UiComponentDefinition::default()
            },
        )]),
    );
    let selection = UiDesignerSelectionModel::single("grandchild".to_string());
    assert_eq!(selected_hierarchy_index(&root_document, &selection), 2);
    assert_eq!(hierarchy_node_id_at(&root_document, 0), Some("root"));
    assert_eq!(hierarchy_node_id_at(&root_document, 3), Some("last"));
    assert_eq!(hierarchy_node_id_at(&root_document, 4), None);
    assert_eq!(build_hierarchy_items(&root_document, Some("last")).len(), 4);

    let component_document = document(
        None,
        BTreeMap::from([
            (
                "second".to_string(),
                UiComponentDefinition {
                    root: node("second_root", Vec::new()),
                    ..UiComponentDefinition::default()
                },
            ),
            (
                "first".to_string(),
                UiComponentDefinition {
                    root: node("first_root", vec![node("first_child", Vec::new())]),
                    ..UiComponentDefinition::default()
                },
            ),
        ]),
    );
    assert_eq!(
        hierarchy_node_id_at(&component_document, 0),
        Some("first_root")
    );
    assert_eq!(
        hierarchy_node_id_at(&component_document, 1),
        Some("first_child")
    );
    assert_eq!(
        hierarchy_node_id_at(&component_document, 2),
        Some("second_root")
    );
}

#[test]
fn optimization_batch_20260826ai_editor23_hierarchy_uses_direct_streaming_traversal() {
    let source = include_str!("../hierarchy_projection.rs");
    let navigation = include_str!("../navigation_state.rs");
    let traversal = source
        .split("pub(super) fn selected_hierarchy_index")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn selection_for_node").next())
        .expect("hierarchy streaming traversal implementation");
    let selection = navigation
        .split("pub fn select_hierarchy_index")
        .nth(1)
        .and_then(|body| body.split("pub fn select_preview_index").next())
        .expect("hierarchy selection implementation");

    assert!(source.contains("use std::ops::ControlFlow;"));
    assert!(traversal.contains("visit_hierarchy_nodes(document"));
    assert!(traversal.contains("visit_node(&child.node"));
    assert!(traversal.contains("ControlFlow::Break"));
    assert!(!traversal.contains("document.node("));
    assert!(!source.contains("fn hierarchy_node_ids"));
    assert!(selection.contains("hierarchy_node_id_at(&self.last_valid_document, index)"));
    assert!(!selection.contains(".into_iter()"));
    assert!(!selection.contains(".nth(index)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ai_editor23_hierarchy_streaming_traversal_performance_evidence() {
    let root = node(
        "hierarchy_root_with_long_identity",
        (1..NODE_COUNT)
            .map(|index| {
                node(
                    format!("hierarchy_node_with_long_identity_{index:05}"),
                    Vec::new(),
                )
            })
            .collect(),
    );
    let document = document(Some(root), BTreeMap::new());
    let target = format!("hierarchy_node_with_long_identity_{:05}", NODE_COUNT - 1);
    let selection = UiDesignerSelectionModel::single(target.clone());
    assert_eq!(
        legacy_selected_hierarchy_index(&document, &target),
        selected_hierarchy_index(&document, &selection)
    );

    let mut cloned_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut streaming_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_selected_hierarchy_index(
                black_box(&document),
                black_box(&target),
            ));
            cloned_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(selected_hierarchy_index(
                black_box(&document),
                black_box(&selection),
            ));
            streaming_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(selected_hierarchy_index(
                black_box(&document),
                black_box(&selection),
            ));
            streaming_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_selected_hierarchy_index(
                black_box(&document),
                black_box(&target),
            ));
            cloned_samples.push(started.elapsed());
        }
    }

    let cloned_p95 = percentile_95(&mut cloned_samples);
    let streaming_p95 = percentile_95(&mut streaming_samples);
    println!(
        "EDITOR23_HIERARCHY_STREAMING_TRAVERSAL_BENCH_V1 \
         nodes={NODE_COUNT} legacy_node_id_clones={NODE_COUNT} streaming_node_id_clones=0 \
         stable_depth_first_order=true cloned_p95_ns={} streaming_p95_ns={}",
        cloned_p95.as_nanos(),
        streaming_p95.as_nanos(),
    );
    assert!(
        streaming_p95.as_nanos() * 100 <= cloned_p95.as_nanos() * 60,
        "streaming hierarchy P95 {:?} exceeded 60% of cloned repeated-lookup P95 {:?}",
        streaming_p95,
        cloned_p95,
    );
}
