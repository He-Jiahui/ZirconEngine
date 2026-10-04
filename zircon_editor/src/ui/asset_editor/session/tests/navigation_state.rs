use super::UiAssetEditorSession;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const OUTLINE_CACHE_LAYOUT: &str = r#"[asset]
kind = "layout"
id = "editor.test.outline_cache"
version = 1
display_name = "Outline Cache"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
"#;

#[test]
fn source_generation_reuses_the_outline_across_presentation_and_navigation() {
    let route = UiAssetEditorRoute::new(
        "editor.test.outline_cache",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session =
        UiAssetEditorSession::from_source(route, OUTLINE_CACHE_LAYOUT, UiSize::new(640.0, 360.0))
            .expect("session");

    assert_eq!(session.source_outline_build_count(), 1);
    assert_eq!(session.source_outline_total_build_count(), 1);
    assert!(session.source_outline_caches_share_index());
    let initial_presentation = session.pane_presentation();
    assert_eq!(initial_presentation.source_outline_items.len(), 1);
    let root_line = session
        .source_outline_index()
        .entry_for_node("root")
        .expect("root outline entry")
        .line as usize;
    session.select_source_line(root_line).expect("source line");
    let _ = session.pane_presentation();
    assert_eq!(session.source_outline_build_count(), 1);
    assert_eq!(session.source_outline_total_build_count(), 1);

    session
        .source_buffer
        .replace(format!("{OUTLINE_CACHE_LAYOUT}\n# source generation one"));
    session
        .select_source_line(root_line)
        .expect("revised source line");
    assert_eq!(session.source_outline_build_count(), 2);
    assert_eq!(session.source_outline_total_build_count(), 2);
    assert!(!session.source_outline_caches_share_index());

    let document = session.last_valid_document.clone();
    session
        .apply_valid_document(document)
        .expect("document refresh");
    session
        .select_source_line(root_line)
        .expect("refreshed source line");
    assert_eq!(session.source_outline_build_count(), 3);
    assert_eq!(session.source_outline_total_build_count(), 3);
    assert!(session.source_outline_caches_share_index());

    session
        .source_buffer
        .replace(format!("{OUTLINE_CACHE_LAYOUT}\n# invalid source draft"));
    session.diagnostics.push("invalid source draft".to_string());
    let _ = session.pane_presentation();
    assert_eq!(session.source_outline_build_count(), 3);
    assert_eq!(session.source_outline_total_build_count(), 3);
}

#[test]
fn pane_presentation_is_value_equivalent_without_a_session_mutation() {
    let route = UiAssetEditorRoute::new(
        "editor.test.presentation_equivalence",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let session =
        UiAssetEditorSession::from_source(route, OUTLINE_CACHE_LAYOUT, UiSize::new(640.0, 360.0))
            .expect("session");

    let first = session.pane_presentation();
    let repeated = session.pane_presentation();

    assert_eq!(repeated, first);
}
