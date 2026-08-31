use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::{
    layout::UiSize,
    template::{UiAssetDocument, UiAssetKind},
};

use super::*;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute};

const TARGET_P95_PERCENT: u128 = 70;

const WRAPPABLE_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "editor.test.owned_wrap_document"
version = 1
display_name = "Owned Wrap Document"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "leaf" }]

[nodes.leaf]
kind = "native"
type = "Label"
control_id = "Leaf"
props = { text = "Leaf" }
"#;

#[test]
fn optimization_batch_20260828ht_editor_preserves_wrap_and_unwrap_behavior() {
    let mut session = benchmark_session();
    let original = session.last_valid_document.clone();
    session.selection = selection_for_node(&session.last_valid_document, "leaf");

    assert!(session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap leaf"));
    let wrapper_id = session
        .selection
        .primary_node_id
        .clone()
        .expect("wrapper selection");
    assert_ne!(wrapper_id, "leaf");

    assert!(session.unwrap_selected_node().expect("unwrap wrapper"));
    assert_eq!(session.selection.primary_node_id.as_deref(), Some("leaf"));
    assert_eq!(session.last_valid_document, original);
}

#[test]
fn optimization_batch_20260828ht_editor_moves_wrap_documents_into_command_handoff() {
    let source = include_str!("../ui_asset_editor_session.rs");
    let wrap = source
        .split("pub fn wrap_selected_node_with")
        .nth(1)
        .and_then(|body| body.split("pub fn unwrap_selected_node").next())
        .expect("wrap implementation");
    let unwrap = source
        .split("pub fn unwrap_selected_node")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn ensure_editable_source").next())
        .expect("unwrap implementation");

    assert!(
        wrap.contains("apply_document_edit_with_tree_edit_and_selection(\n            document,")
    );
    assert_eq!(wrap.matches("document.clone()").count(), 1);
    assert!(
        unwrap.contains("apply_document_edit_with_tree_edit_and_selection(\n            document,")
    );
    assert_eq!(unwrap.matches("document.clone()").count(), 1);
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ht_editor_owned_wrap_document_handoff_benchmark() {
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
        "EDITOR212_OWNED_WRAP_DOCUMENT_HANDOFF_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
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
            "editor.test.owned_wrap_document",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        WRAPPABLE_LAYOUT,
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
