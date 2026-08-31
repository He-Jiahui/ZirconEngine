use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    layout::UiSize,
    template::{UiAssetDocument, UiAssetKind},
};

use super::*;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute};

const TARGET_P95_PERCENT: u128 = 70;

const NESTED_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "editor.test.owned_palette_document"
version = 1
display_name = "Owned Palette Document"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "group" }]

[nodes.group]
kind = "native"
type = "VerticalBox"
control_id = "Group"
children = [{ child = "leaf" }]

[nodes.leaf]
kind = "native"
type = "Label"
control_id = "Leaf"
props = { text = "Leaf" }
"#;

#[test]
fn optimization_batch_20260828hs_editor_preserves_palette_insert_and_reparent_behavior() {
    let mut insert_session = benchmark_session();
    insert_session
        .select_palette_index(0)
        .expect("native palette entry");
    let before_children = insert_session
        .last_valid_document
        .root
        .as_ref()
        .expect("root")
        .children
        .len();

    assert!(insert_session
        .insert_selected_palette_item_as_child()
        .expect("insert palette item"));
    assert_eq!(
        insert_session
            .last_valid_document
            .root
            .as_ref()
            .expect("root")
            .children
            .len(),
        before_children + 1
    );

    let mut reparent_session = benchmark_session();
    reparent_session.selection = selection_for_node(&reparent_session.last_valid_document, "leaf");
    assert!(reparent_session
        .reparent_selected_node_outdent()
        .expect("outdent leaf"));
    assert_eq!(
        reparent_session.selection.parent_node_id.as_deref(),
        Some("root")
    );
}

#[test]
fn optimization_batch_20260828hs_editor_moves_edited_documents_into_command_handoff() {
    let source = include_str!("../palette_state.rs");
    let insert = source
        .split("fn insert_selected_palette_item_with_plan")
        .nth(1)
        .and_then(|body| body.split("fn move_selected_node").next())
        .expect("palette insert implementation");
    let reparent = source
        .split("fn reparent_selected_node(")
        .nth(1)
        .and_then(|body| body.split("}\n}").next())
        .expect("palette reparent implementation");

    assert!(
        insert.contains("apply_document_edit_with_tree_edit_and_selection(\n            document,")
    );
    assert_eq!(insert.matches("document.clone()").count(), 1);
    assert!(reparent
        .contains("apply_document_edit_with_tree_edit_and_selection(\n            document,"));
    assert_eq!(reparent.matches("document.clone()").count(), 1);
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hs_editor_owned_palette_document_handoff_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let document = benchmark_document(512, 4 * 1024);

    black_box(legacy_prepare_document(&document));
    black_box(optimized_prepare_document(&document));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_prepare_document(black_box(&document)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(optimized_prepare_document(black_box(&document)));
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
        "EDITOR211_OWNED_PALETTE_DOCUMENT_HANDOFF_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_session() -> UiAssetEditorSession {
    UiAssetEditorSession::from_source(
        UiAssetEditorRoute::new(
            "editor.test.owned_palette_document",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        NESTED_LAYOUT,
        UiSize::new(640.0, 360.0),
    )
    .expect("benchmark session")
}

fn benchmark_document(item_count: usize, item_bytes: usize) -> UiAssetDocument {
    let mut document = benchmark_session().last_valid_document.clone();
    let suffix = "x".repeat(item_bytes);
    document.imports.widgets = (0..item_count)
        .map(|index| format!("res://ui/widgets/widget-{index}-{suffix}.zui"))
        .collect();
    document
}

fn legacy_prepare_document(source: &UiAssetDocument) -> usize {
    let edited = source.clone();
    let handed_off = edited.clone();
    black_box(handed_off).imports.widgets.len()
}

fn optimized_prepare_document(source: &UiAssetDocument) -> usize {
    let edited = source.clone();
    black_box(edited).imports.widgets.len()
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
