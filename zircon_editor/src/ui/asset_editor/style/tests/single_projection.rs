use std::{collections::BTreeMap, hint::black_box, time::Instant};

use super::*;
use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiAssetImports, UiAssetKind, UiChildMount, UiStyleDeclarationBlock, UiStyleRule,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826au_matched_style_path_preserves_host_and_descendant_matching() {
    let mut document = document_with_depth(3);
    document.stylesheets = vec![UiStyleSheet {
        id: "local".to_string(),
        rules: vec![UiStyleRule {
            id: Some("target-hover".to_string()),
            selector: ":host > Panel .target:hover".to_string(),
            set: UiStyleDeclarationBlock::default(),
        }],
    }];
    let states = vec!["hover".to_string()];

    let path = document_style_match_path(&document, "target", &states).expect("target path");
    assert_eq!(path.len(), 3);
    assert!(path[0].is_host);
    assert!(!path[1].is_host);
    assert!(!path[2].is_host);
    assert_eq!(path[0].component, "Root");
    assert_eq!(path[1].component, "Panel");
    assert_eq!(path[2].component, "Button");

    let matched = matched_style_rule_entries(&document, &BTreeMap::new(), "target", &states);
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].selector, ":host > Panel .target:hover");
    assert!(matched_style_rule_entries(&document, &BTreeMap::new(), "missing", &states).is_empty());
}

#[test]
fn optimization_batch_20260826au_matched_style_path_uses_single_projection() {
    let source = include_str!("../matched_rule_inspection.rs");
    let entry = bounded_source(
        source,
        "pub(crate) fn matched_style_rule_entries(",
        "fn collect_matching_rules(",
    );
    let path = bounded_source(
        source,
        "fn document_style_match_path",
        "pub(crate) fn selector_component_name",
    );

    assert!(entry.contains("document_style_match_path"));
    assert!(!entry.contains(".map(|(index"));
    assert!(!source.contains("fn document_node_path"));
    assert!(!path.contains("Vec<(&"));
    assert_eq!(path.matches("path.push(StyleMatchNode").count(), 1);
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826au_matched_style_path_single_projection_p95() {
    const PATH_DEPTH: usize = 2_048;
    const BUILDS: usize = 128;
    let document = document_with_depth(PATH_DEPTH);
    let states = vec!["hover".to_string(), "focus".to_string()];
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_document_style_match_path(&document, "target", &states)
                    .map(|path| path.len())
                    .unwrap_or_default()
            }));
            optimized_ns.push(measure_ns(BUILDS, || {
                document_style_match_path(&document, "target", &states)
                    .map(|path| path.len())
                    .unwrap_or_default()
            }));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || {
                document_style_match_path(&document, "target", &states)
                    .map(|path| path.len())
                    .unwrap_or_default()
            }));
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_document_style_match_path(&document, "target", &states)
                    .map(|path| path.len())
                    .unwrap_or_default()
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(17),
        "single-projection match path P95 must be at least 15% below tuple-path remapping: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_MATCHED_STYLE_PATH_SINGLE_PROJECTION_BENCH_V1 path_depth={PATH_DEPTH} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_path_allocations_per_sample={} optimized_path_allocations_per_sample={} legacy_tuple_entries_written_per_sample={} optimized_tuple_entries_written_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        BUILDS * 2,
        BUILDS,
        PATH_DEPTH * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn document_with_depth(depth: usize) -> UiAssetDocument {
    assert!(depth >= 1);
    let mut child = UiNodeDefinition {
        node_id: "target".to_string(),
        widget_type: Some("Button".to_string()),
        classes: vec!["target".to_string()],
        ..UiNodeDefinition::default()
    };
    for index in (1..depth.saturating_sub(1)).rev() {
        child = UiNodeDefinition {
            node_id: if index == 1 {
                "panel".to_string()
            } else {
                format!("node-{index}")
            },
            widget_type: Some(if index == 1 { "Panel" } else { "Container" }.to_string()),
            children: vec![UiChildMount {
                node: child,
                ..UiChildMount::default()
            }],
            ..UiNodeDefinition::default()
        };
    }
    let root = if depth == 1 {
        UiNodeDefinition {
            node_id: "target".to_string(),
            widget_type: Some("Root".to_string()),
            classes: vec!["target".to_string()],
            ..UiNodeDefinition::default()
        }
    } else {
        UiNodeDefinition {
            node_id: "root".to_string(),
            widget_type: Some("Root".to_string()),
            children: vec![UiChildMount {
                node: child,
                ..UiChildMount::default()
            }],
            ..UiNodeDefinition::default()
        }
    };

    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Layout,
            id: "layout.matched-path-test".to_string(),
            version: 1,
            display_name: String::new(),
        },
        imports: UiAssetImports::default(),
        tokens: BTreeMap::new(),
        root: Some(root),
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}

fn legacy_document_style_match_path<'a>(
    document: &'a UiAssetDocument,
    node_id: &str,
    active_states: &'a [String],
) -> Option<Vec<StyleMatchNode<'a>>> {
    fn visit<'a>(
        node: &'a UiNodeDefinition,
        target: &str,
        path: &mut Vec<(&'a str, &'a UiNodeDefinition)>,
    ) -> bool {
        path.push((node.node_id.as_str(), node));
        if node.node_id == target {
            return true;
        }
        for child in &node.children {
            if visit(&child.node, target, path) {
                return true;
            }
        }
        let _ = path.pop();
        false
    }

    let root = document.root.as_ref()?;
    let mut path = Vec::new();
    if !visit(root, node_id, &mut path) {
        return None;
    }
    Some(
        path.iter()
            .enumerate()
            .map(|(index, (_, node))| StyleMatchNode {
                component: selector_component_name(node),
                control_id: node.control_id.as_deref(),
                classes: &node.classes,
                is_host: index == 0,
                states: active_states,
            })
            .collect(),
    )
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
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
