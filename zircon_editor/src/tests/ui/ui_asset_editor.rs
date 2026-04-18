use serde_json::json;

use crate::{
    UiAssetEditorMode, UiAssetEditorReflectionModel, UiAssetEditorRoute, UiDesignerSelectionModel,
    UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel, UiAssetEditorSession,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
};
use zircon_ui::{UiAssetKind, UiAssetLoader, UiDocumentCompiler, UiSize};

const UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/ui_asset_editor.ui.toml"
));
const UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/editor_widgets.ui.toml"
));
const UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/theme/editor_base.ui.toml"
));

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

#[test]
fn ui_asset_editor_bootstrap_route_points_to_shared_layout_asset() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );

    assert_eq!(route.asset_id, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID);
    assert_eq!(
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID,
        "editor.ui_asset_editor"
    );
}

#[test]
fn ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports() {
    let layout = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");
    let widget = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE, widget.clone())
        .expect("register bootstrap widget import");
    compiler
        .register_widget_import(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE, widget)
        .expect("register bootstrap section card import");
    compiler
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style)
        .expect("register bootstrap style import");

    let compiled = compiler
        .compile(&layout)
        .expect("compile bootstrap editor layout");
    let root = &compiled.template_instance().root;

    assert_eq!(compiled.asset.id, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID);
    assert_eq!(root.component.as_deref(), Some("VerticalBox"));
    assert!(root.children.len() >= 2);
}

#[test]
fn ui_asset_editor_bootstrap_assets_open_in_session_after_import_hydration() {
    let widget = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID,
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap session");

    assert!(
        !session.diagnostics().is_empty(),
        "bootstrap session should report missing imports before hydration"
    );

    session
        .register_widget_import(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE, widget.clone())
        .expect("hydrate bootstrap widget import");
    session
        .register_widget_import(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE, widget)
        .expect("hydrate bootstrap section card import");
    session
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style)
        .expect("hydrate bootstrap style import");

    assert!(
        session.diagnostics().is_empty(),
        "bootstrap session should compile once imports are hydrated"
    );
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert_eq!(pane.asset_id, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID);
    assert!(pane.preview_canvas_items.len() >= 3);
}
