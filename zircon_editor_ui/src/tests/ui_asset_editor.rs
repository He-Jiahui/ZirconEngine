use serde_json::json;

use crate::{
    UiAssetEditorMode, UiAssetEditorReflectionModel, UiAssetEditorRoute, UiDesignerSelectionModel,
    UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel,
};
use zircon_ui::UiAssetKind;

#[test]
fn ui_asset_editor_reflection_model_tracks_source_selection_and_style_state() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/editor/ui_asset_editor.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let selection = UiDesignerSelectionModel::single("designer_root")
        .with_parent("surface_root")
        .with_mount("content")
        .with_sibling("designer_toolbar");
    let style_inspector = UiStyleInspectorReflectionModel::for_node("designer_root")
        .with_class("editor-shell")
        .with_active_pseudo_state("hover")
        .with_inline_override("self.background.color", json!("#20242c"))
        .with_matched_rule(UiMatchedStyleRuleReflection::new(
            "editor_base",
            ".editor-shell:hover",
            20,
            0,
        ));

    let model = UiAssetEditorReflectionModel::new(route.clone(), "UI Asset Editor")
        .with_source_dirty(true)
        .with_undo_state(true, false)
        .with_preview_available(true)
        .with_last_error("expected a TOML table")
        .with_selection(selection.clone())
        .with_style_inspector(style_inspector.clone());

    assert_eq!(model.route, route);
    assert_eq!(model.display_name, "UI Asset Editor");
    assert!(model.source_dirty);
    assert!(model.can_undo);
    assert!(!model.can_redo);
    assert!(model.preview_available);
    assert_eq!(model.last_error.as_deref(), Some("expected a TOML table"));
    assert_eq!(model.selection, selection);
    assert_eq!(model.style_inspector, style_inspector);
}

#[test]
fn ui_designer_selection_model_reports_multi_selection_only_for_sibling_groups() {
    let single = UiDesignerSelectionModel::single("root");
    assert!(!single.is_multi_select());

    let multi = UiDesignerSelectionModel::single("root")
        .with_sibling("toolbar")
        .with_sibling("content");
    assert!(multi.is_multi_select());
    assert_eq!(
        multi.sibling_node_ids,
        vec![
            "root".to_string(),
            "toolbar".to_string(),
            "content".to_string()
        ]
    );
}
