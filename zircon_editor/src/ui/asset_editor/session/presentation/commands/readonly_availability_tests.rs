use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

use super::*;
use crate::ui::asset_editor::tree::tree_editing::{
    move_selected_node, reparent_selected_node, unwrap_selected_node, wrap_selected_node,
};
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiDesignerSelectionModel};

const TARGET_P95_PERCENT: u128 = 70;

const COMMAND_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "editor.test.readonly_command_availability"
version = 1
display_name = "Readonly Command Availability"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "first" }, { child = "container" }, { child = "last" }]

[nodes.first]
kind = "native"
type = "Label"
control_id = "First"
props = { text = "First" }

[nodes.container]
kind = "native"
type = "VerticalBox"
control_id = "Container"
children = [{ child = "inner" }]

[nodes.inner]
kind = "native"
type = "Label"
control_id = "Inner"
props = { text = "Inner" }

[nodes.last]
kind = "native"
type = "Label"
control_id = "Last"
props = { text = "Last" }
"#;

#[test]
fn optimization_batch_20260828hu_editor_readonly_availability_matches_legacy_tree_edits() {
    let document = benchmark_session(COMMAND_LAYOUT).last_valid_document;

    for node_id in ["first", "container", "inner", "last", "root", "missing"] {
        let selection = UiDesignerSelectionModel::single(node_id);
        let optimized = readonly_tree_command_availability(&document, &selection);
        let legacy = legacy_tree_command_availability(&document, &selection);
        assert_eq!(optimized, legacy, "availability mismatch for {node_id}");
    }

    let empty = UiDesignerSelectionModel::default();
    assert_eq!(
        readonly_tree_command_availability(&document, &empty),
        legacy_tree_command_availability(&document, &empty)
    );
}

#[test]
fn optimization_batch_20260828hu_editor_command_projection_uses_readonly_tree_queries() {
    let source = include_str!("../commands.rs");
    let command_availability = source
        .split("pub(super) fn command_availability")
        .nth(1)
        .and_then(|body| body.split("fn readonly_tree_command_availability").next())
        .expect("command availability implementation");

    assert!(command_availability.contains("readonly_tree_command_availability("));
    assert!(!command_availability.contains("can_apply_tree_document_edit"));
    assert!(!source.contains("fn can_apply_tree_document_edit"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hu_editor_readonly_command_availability_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let document = benchmark_document(512, 4 * 1024);
    let selection = UiDesignerSelectionModel::single("leaf_256");

    black_box(legacy_tree_command_availability(&document, &selection));
    black_box(readonly_tree_command_availability(&document, &selection));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_tree_command_availability(
                    black_box(&document),
                    black_box(&selection),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(readonly_tree_command_availability(
                    black_box(&document),
                    black_box(&selection),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR213_READONLY_COMMAND_AVAILABILITY_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_tree_command_availability(
    document: &UiAssetDocument,
    selection: &UiDesignerSelectionModel,
) -> UiAssetTreeCommandAvailability {
    UiAssetTreeCommandAvailability {
        can_move_up: legacy_apply(document, |candidate| {
            move_selected_node(candidate, selection, UiTreeMoveDirection::Up)
        }),
        can_move_down: legacy_apply(document, |candidate| {
            move_selected_node(candidate, selection, UiTreeMoveDirection::Down)
        }),
        can_reparent_into_previous: legacy_apply(document, |candidate| {
            reparent_selected_node(candidate, selection, UiTreeReparentDirection::IntoPrevious)
                .is_some()
        }),
        can_reparent_into_next: legacy_apply(document, |candidate| {
            reparent_selected_node(candidate, selection, UiTreeReparentDirection::IntoNext)
                .is_some()
        }),
        can_reparent_outdent: legacy_apply(document, |candidate| {
            reparent_selected_node(candidate, selection, UiTreeReparentDirection::Outdent).is_some()
        }),
        can_wrap_in_vertical_box: legacy_apply(document, |candidate| {
            wrap_selected_node(candidate, selection, "VerticalBox").is_some()
        }),
        can_unwrap: legacy_apply(document, |candidate| {
            unwrap_selected_node(candidate, selection).is_some()
        }),
    }
}

fn legacy_apply(
    document: &UiAssetDocument,
    edit: impl FnOnce(&mut UiAssetDocument) -> bool,
) -> bool {
    let mut document = document.clone();
    edit(&mut document)
}

fn benchmark_session(source: &str) -> UiAssetEditorSession {
    UiAssetEditorSession::from_source(
        UiAssetEditorRoute::new(
            "editor.test.readonly_command_availability",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        source,
        UiSize::new(640.0, 360.0),
    )
    .expect("benchmark session")
}

fn benchmark_document(item_count: usize, item_bytes: usize) -> UiAssetDocument {
    let suffix = "x".repeat(item_bytes);
    let children = (0..item_count)
        .map(|index| format!("{{ child = \"leaf_{index}\" }}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut source = format!(
        "[asset]\nkind = \"layout\"\nid = \"editor.test.readonly_command_availability\"\nversion = 1\ndisplay_name = \"Readonly Command Availability\"\n\n[root]\nnode = \"root\"\n\n[nodes.root]\nkind = \"native\"\ntype = \"VerticalBox\"\ncontrol_id = \"Root\"\nchildren = [{children}]\n"
    );
    for index in 0..item_count {
        source.push_str(&format!(
            "\n[nodes.leaf_{index}]\nkind = \"native\"\ntype = \"Label\"\ncontrol_id = \"Leaf{index}\"\nprops = {{ text = \"{suffix}\" }}\n"
        ));
    }
    benchmark_session(&source).last_valid_document
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
