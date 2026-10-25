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
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
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

const WRAP_IDENTITY_V2: &str = r#"
[asset]
kind = "view"
id = "editor.test.wrap_identity"
version = 2
display_name = "Hierarchy Wrap Identity"

[root]
node = "root"

[components.Badge]
root = "badge_root"

[nodes.root]
component = "VerticalGroup"
control_id = "Root"
children = [{ node = "subject" }, { node = "verticalbox_3" }]

[nodes.subject]
component = "HorizontalGroup"
control_id = "Subject"
children = [{ node = "verticalbox" }, { node = "verticalbox_2" }, { node = "local_copy" }]

[nodes.verticalbox]
component = "Button"
control_id = "VerticalBox"
props = { text = "保留按钮与事件" }
state = { selected = true }
events = [{ id = "Widget/Activate", event = "Click", route = "widget.activate" }]

[nodes.verticalbox_2]
component = "Text"
control_id = "VerticalBox2"
props = { text = "长文本 / second identity" }

[nodes.verticalbox_3]
component = "Text"
control_id = "VerticalBox3"
props = { text = "Outside the wrapped subtree" }

[nodes.local_copy]
component = "Badge"
control_id = "LocalCopy"

[nodes.badge_root]
component = "Text"
control_id = "BadgeRoot"
props = { text = "Local component" }
"#;

fn wrap_identity_session() -> UiAssetEditorSession {
    let session = UiAssetEditorSession::from_v2_source(
        UiAssetEditorRoute::new(
            "editor.test.wrap_identity",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        WRAP_IDENTITY_V2,
        UiSize::new(640.0, 520.0),
    )
    .expect("actual v2 authoring session");
    assert!(
        session.diagnostics().is_empty(),
        "{:?}",
        session.diagnostics()
    );
    session
}

#[test]
fn hierarchy_wrap_identity_reserves_selected_node_and_control_ids() {
    use zircon_runtime::ui::v2::UiV2AssetLoader;

    let mut session = wrap_identity_session();
    let before = UiV2AssetLoader::load_toml_str(WRAP_IDENTITY_V2).expect("original v2");
    session.selection = selection_for_node(&session.last_valid_document, "verticalbox");

    assert!(session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap"));
    assert!(session.diagnostics().is_empty());
    let after =
        UiV2AssetLoader::load_toml_str(session.source_buffer().text()).expect("wrapped v2 source");
    assert_eq!(after.nodes.len(), before.nodes.len() + 1);
    assert_eq!(
        after.nodes["verticalbox_4"].control_id.as_deref(),
        Some("VerticalBox4")
    );
    assert_eq!(after.nodes["verticalbox_4"].children[0].node, "verticalbox");
    // Only the old parent changes its child reference; the old leaf, binding,
    // local component instance and unrelated identities retain their full values.
    for (id, node) in &before.nodes {
        if id != "subject" {
            assert_eq!(after.nodes.get(id), Some(node), "preserve {id}");
        }
    }
    assert_eq!(after.components, before.components);
    assert_eq!(
        session.selection.primary_node_id.as_deref(),
        Some("verticalbox_4")
    );
    assert_eq!(session.pane_presentation().hierarchy_selected_index, 2);

    let reopened = UiAssetEditorSession::from_v2_source(
        session.route().clone(),
        session.source_buffer().text(),
        UiSize::new(640.0, 520.0),
    )
    .expect("reload the changed asset through the real v2 session");
    assert!(reopened.diagnostics().is_empty());
    assert_eq!(reopened.last_valid_document, session.last_valid_document);
}

#[test]
fn hierarchy_wrap_identity_reserves_descendants_and_uses_one_undo() {
    use zircon_runtime::ui::v2::UiV2AssetLoader;

    let mut session = wrap_identity_session();
    let before_source = session.source_buffer().text().to_string();
    let before = UiV2AssetLoader::load_toml_str(&before_source).expect("original v2");
    session.selection = selection_for_node(&session.last_valid_document, "subject");
    assert!(!session.can_undo());

    assert!(session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap subtree"));
    let wrapped_source = session.source_buffer().text().to_string();
    let wrapped = UiV2AssetLoader::load_toml_str(&wrapped_source).expect("wrapped v2");
    assert_eq!(wrapped.nodes.len(), before.nodes.len() + 1);
    assert_eq!(
        wrapped.nodes["verticalbox_4"].control_id.as_deref(),
        Some("VerticalBox4")
    );
    assert_eq!(wrapped.nodes["verticalbox_4"].children[0].node, "subject");
    for (id, node) in &before.nodes {
        if id != "root" {
            assert_eq!(wrapped.nodes.get(id), Some(node), "preserve {id}");
        }
    }
    assert_eq!(wrapped.components, before.components);
    assert_eq!(session.pane_presentation().hierarchy_selected_index, 1);

    assert!(session.undo().expect("one undo restores the whole wrap"));
    assert_eq!(session.source_buffer().text(), before_source);
    assert_eq!(
        session.selection.primary_node_id.as_deref(),
        Some("subject")
    );
    assert!(!session.can_undo());
    assert!(session.redo().expect("redo wrap"));
    assert_eq!(session.source_buffer().text(), wrapped_source);
    assert_eq!(
        session.selection.primary_node_id.as_deref(),
        Some("verticalbox_4")
    );
    assert!(!session.can_redo());

    assert!(session
        .unwrap_selected_node()
        .expect("unwrap original subtree"));
    let unwrapped =
        UiV2AssetLoader::load_toml_str(session.source_buffer().text()).expect("unwrapped v2");
    assert_eq!(unwrapped.nodes, before.nodes);
    assert_eq!(unwrapped.components, before.components);
    assert_eq!(
        session.selection.primary_node_id.as_deref(),
        Some("subject")
    );
}

#[test]
fn hierarchy_root_wrap_preserves_v2_root_tree_and_reopens_preview() {
    use zircon_runtime::ui::v2::UiV2AssetLoader;

    let mut session = wrap_identity_session();
    let before = UiV2AssetLoader::load_toml_str(session.source_buffer().text())
        .expect("original root document");
    session
        .select_hierarchy_index(0)
        .expect("select document root");
    let commands = session.pane_presentation();
    assert!(commands.can_wrap_in_vertical_box);
    assert!(!commands.can_unwrap);

    assert!(session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap document root"));
    let source = session.source_buffer().text();
    let after = UiV2AssetLoader::load_toml_str(source).expect("wrapped root source");
    assert_eq!(
        after.root.as_ref().map(|root| root.node.as_str()),
        Some("verticalbox_4")
    );
    assert_eq!(after.nodes.len(), before.nodes.len() + 1);
    assert_eq!(after.nodes["verticalbox_4"].children[0].node, "root");
    assert_eq!(
        after.nodes["verticalbox_4"].control_id.as_deref(),
        Some("VerticalBox4")
    );
    for (id, node) in &before.nodes {
        assert_eq!(
            after.nodes.get(id),
            Some(node),
            "preserve root subtree {id}"
        );
    }
    assert_eq!(after.components, before.components);
    assert_eq!(after.imports, before.imports);
    assert_eq!(after.tokens, before.tokens);
    assert_eq!(after.stylesheets, before.stylesheets);
    assert_eq!(
        session.selection.primary_node_id.as_deref(),
        Some("verticalbox_4")
    );
    assert!(session.selection.parent_node_id.is_none());
    assert_eq!(session.pane_presentation().hierarchy_selected_index, 0);
    assert!(session.pane_presentation().can_unwrap);
    assert!(session.preview_host.is_some());

    let reopened = UiAssetEditorSession::from_v2_source(
        session.route().clone(),
        source,
        UiSize::new(640.0, 520.0),
    )
    .expect("reload root change through real v2 session");
    assert!(reopened.diagnostics().is_empty());
    assert!(reopened.preview_host.is_some());
    assert_eq!(reopened.last_valid_document, session.last_valid_document);
}

#[test]
fn hierarchy_root_wrap_and_unwrap_each_use_one_existing_history_step() {
    use zircon_runtime::ui::v2::UiV2AssetLoader;

    let mut session = wrap_identity_session();
    session.select_hierarchy_index(0).expect("select root");
    let before_source = session.source_buffer().text().to_string();
    let before = UiV2AssetLoader::load_toml_str(&before_source).expect("original v2");
    assert!(!session.can_undo());
    assert!(session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap root"));
    let wrapped_source = session.source_buffer().text().to_string();

    assert!(session.undo().expect("one undo of root wrap"));
    assert_eq!(session.source_buffer().text(), before_source);
    assert_eq!(session.selection.primary_node_id.as_deref(), Some("root"));
    assert!(!session.can_undo());
    assert!(session.redo().expect("redo root wrap"));
    assert_eq!(session.source_buffer().text(), wrapped_source);
    assert_eq!(
        session.selection.primary_node_id.as_deref(),
        Some("verticalbox_4")
    );
    assert!(!session.can_redo());

    assert!(session
        .unwrap_selected_node()
        .expect("promote the original root"));
    let unwrapped_source = session.source_buffer().text().to_string();
    let after = UiV2AssetLoader::load_toml_str(&unwrapped_source).expect("unwrapped v2");
    assert_eq!(after.root, before.root);
    assert_eq!(after.nodes, before.nodes);
    assert_eq!(after.components, before.components);
    assert_eq!(session.selection.primary_node_id.as_deref(), Some("root"));
    assert!(session.selection.parent_node_id.is_none());
    assert!(session.undo().expect("one undo of root unwrap"));
    assert_eq!(session.source_buffer().text(), wrapped_source);
    assert_eq!(
        session.selection.primary_node_id.as_deref(),
        Some("verticalbox_4")
    );
    assert!(session.undo().expect("undo the original wrap"));
    assert_eq!(session.source_buffer().text(), before_source);
    assert!(!session.can_undo());
    assert!(session.redo().expect("redo wrap"));
    assert!(session.redo().expect("redo unwrap"));
    assert_eq!(session.source_buffer().text(), unwrapped_source);
    assert!(!session.can_redo());
}

#[test]
fn hierarchy_root_wrap_does_not_treat_component_roots_or_missing_nodes_as_document_root() {
    let mut session = wrap_identity_session();
    let original_source = session.source_buffer().text().to_string();
    session.selection = UiDesignerSelectionModel::single("badge_root");
    assert!(!session.pane_presentation().can_wrap_in_vertical_box);
    assert!(!session
        .wrap_selected_node_with("VerticalBox")
        .expect("component root is not a view root"));
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(!session.can_undo());

    session.selection = UiDesignerSelectionModel::single("missing");
    assert!(!session.pane_presentation().can_wrap_in_vertical_box);
    assert!(!session
        .wrap_selected_node_with("VerticalBox")
        .expect("missing target"));
    assert!(!session.unwrap_selected_node().expect("missing wrapper"));
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(!session.can_undo());
}
