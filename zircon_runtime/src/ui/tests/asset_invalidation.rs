use std::collections::BTreeSet;

use crate::ui::template::{
    collect_invalidation_diagnostics, fingerprint_document, UiAssetLoader, UiInvalidationGraph,
    BROAD_SELECTOR_WARNING_THRESHOLD, LARGE_DOCUMENT_NODE_WARNING_THRESHOLD,
    NON_VIRTUALIZED_SCROLL_CHILD_WARNING_THRESHOLD,
};
use zircon_runtime_interface::ui::template::{
    UiAssetChange, UiAssetDocument, UiAssetFingerprint, UiInvalidationImpact,
    UiInvalidationSnapshot, UiInvalidationStage,
};

const DOC_A: &str = r#"
[asset]
kind = "layout"
id = "editor.cache_test"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "A" }
"#;

const DOC_B: &str = r#"
[asset]
kind = "layout"
id = "editor.cache_test"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "B" }
"#;

#[test]
fn asset_invalidation_graph_maps_document_change_to_rebuild_stages_and_dirty_flags() {
    let previous_document = UiAssetLoader::load_toml_str(DOC_A).unwrap();
    let next_document = UiAssetLoader::load_toml_str(DOC_B).unwrap();
    let previous = snapshot_for_document(&previous_document);
    let next = snapshot_for_document(&next_document);

    let report = UiInvalidationGraph::classify(Some(&previous), &next, &next_document);

    assert!(report.changes.contains(&UiAssetChange::Document));
    assert!(report.stages.contains(&UiInvalidationStage::SourceParse));
    assert!(report.stages.contains(&UiInvalidationStage::DocumentShape));
    assert!(report.stages.contains(&UiInvalidationStage::Layout));
    assert!(report.impact.rebuild_required);
    assert!(report.impact.dirty.layout);
    assert!(report.impact.dirty.hit_test);
    assert!(report.impact.dirty.render);
    assert!(report.impact.projection_dirty);
}

#[test]
fn asset_invalidation_impact_maps_late_stages_to_runtime_dirty_flags() {
    let stages = BTreeSet::from([
        UiInvalidationStage::StyleValue,
        UiInvalidationStage::Interaction,
        UiInvalidationStage::Projection,
    ]);

    let impact = UiInvalidationImpact::from_stages(&stages);

    assert!(impact.dirty.style);
    assert!(impact.dirty.layout);
    assert!(impact.dirty.hit_test);
    assert!(impact.dirty.render);
    assert!(impact.dirty.text);
    assert!(impact.dirty.input);
    assert!(impact.projection_dirty);
    assert!(!impact.rebuild_required);
}

#[test]
fn asset_invalidation_graph_reports_resource_dependency_changes() {
    let document = UiAssetLoader::load_toml_str(DOC_A).unwrap();
    let mut previous = snapshot_for_document(&document);
    let mut next = previous.clone();
    previous.resource_dependencies_revision = UiAssetFingerprint::from_bytes(b"resource-a");
    next.resource_dependencies_revision = UiAssetFingerprint::from_bytes(b"resource-b");

    let report = UiInvalidationGraph::classify(Some(&previous), &next, &document);

    assert_eq!(report.changes, [UiAssetChange::ResourceDependency]);
    assert!(report
        .stages
        .contains(&UiInvalidationStage::ResourceDependency));
    assert!(report.stages.contains(&UiInvalidationStage::Render));
    assert!(report.stages.contains(&UiInvalidationStage::Projection));
    assert!(report.impact.rebuild_required);
    assert!(report.impact.dirty.render);
    assert!(report.impact.projection_dirty);
}

#[test]
fn asset_invalidation_diagnostics_report_broad_selector_pressure() {
    let mut source = String::from(
        r#"
[asset]
kind = "layout"
id = "editor.broad_selector_test"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
"#,
    );
    source.push_str(
        r#"
[[stylesheets]]
id = "host_styles"
"#,
    );
    for _ in 0..BROAD_SELECTOR_WARNING_THRESHOLD {
        source.push_str(
            r#"
[[stylesheets.rules]]
selector = "Label"
set = { self = { color = "accent" } }
"#,
        );
    }

    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let diagnostics = collect_invalidation_diagnostics(&document);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "broad_selector"));
}

#[test]
fn asset_invalidation_diagnostics_report_large_documents() {
    let mut source = String::from(
        r#"
[asset]
kind = "layout"
id = "editor.large_document_test"
version = 1

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
"#,
    );
    for index in 0..LARGE_DOCUMENT_NODE_WARNING_THRESHOLD {
        source.push_str(&format!(
            r#"
[[root.children]]
[root.children.node]
node_id = "node_{index}"
kind = "native"
type = "Label"
"#
        ));
    }

    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let diagnostics = collect_invalidation_diagnostics(&document);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "large_document"));
}

#[test]
fn asset_invalidation_diagnostics_report_non_virtualized_scroll_children() {
    let mut source = String::from(
        r#"
[asset]
kind = "layout"
id = "editor.scroll_pressure_test"
version = 1

[root]
node_id = "scroll_root"
kind = "native"
type = "ScrollableBox"
layout = { container = { kind = "ScrollableBox" } }
"#,
    );
    for index in 0..NON_VIRTUALIZED_SCROLL_CHILD_WARNING_THRESHOLD {
        source.push_str(&format!(
            r#"
[[root.children]]
[root.children.node]
node_id = "row_{index}"
kind = "native"
type = "Label"
"#
        ));
    }

    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let diagnostics = collect_invalidation_diagnostics(&document);

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "non_virtualized_scroll_children"));
}

fn snapshot_for_document(document: &UiAssetDocument) -> UiInvalidationSnapshot {
    UiInvalidationSnapshot {
        document: fingerprint_document(document).unwrap(),
        ..Default::default()
    }
}
