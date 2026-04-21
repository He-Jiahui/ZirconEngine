use super::support::{
    hydrate_bootstrap_imports, register_bootstrap_imports, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML, UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML,
};
use crate::ui::asset_editor::{
    UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
};
use zircon_runtime::ui::template::{UiDocumentCompiler, UiNodeDefinitionKind};
use zircon_runtime::ui::{layout::UiSize, template::UiAssetKind};

#[test]
fn ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports() {
    let layout = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");
    let mut compiler = UiDocumentCompiler::default();
    register_bootstrap_imports(&mut compiler);

    let compiled = compiler
        .compile(&layout)
        .expect("compile bootstrap editor layout");
    let root = &compiled.template_instance().root;

    assert_eq!(
        compiled.asset.id,
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID
    );
    assert_eq!(root.component.as_deref(), Some("VerticalBox"));
    assert!(root.children.len() >= 2);
}

#[test]
fn ui_asset_editor_bootstrap_assets_open_in_session_after_import_hydration() {
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

    hydrate_bootstrap_imports(&mut session);

    assert!(
        session.diagnostics().is_empty(),
        "bootstrap session should compile once imports are hydrated"
    );
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert_eq!(pane.asset_id, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID);
    assert!(pane.preview_canvas_items.len() >= 3);
}

#[test]
fn ui_asset_editor_bootstrap_widget_asset_opens_as_self_hosted_widget_session() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID,
        UiAssetKind::Widget,
        UiAssetEditorMode::Design,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap widget session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(!pane.preview_canvas_items.is_empty());
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("section_card_root")));
}

#[test]
fn ui_asset_editor_bootstrap_style_asset_opens_as_self_hosted_style_session() {
    let route = UiAssetEditorRoute::new(
        UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID,
        UiAssetKind::Style,
        UiAssetEditorMode::Design,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("bootstrap style session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(!pane.preview_available);
    assert_eq!(pane.style_token_items.len(), 4);
    assert_eq!(pane.style_rule_items.len(), 5);
    assert_eq!(pane.asset_id, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID);
}

#[test]
fn ui_asset_editor_bootstrap_layout_uses_shared_toolbar_widget_reference() {
    let layout = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");

    assert!(layout
        .imports
        .widgets
        .iter()
        .any(|reference| reference == UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE));
    let toolbar = layout.node("toolbar").expect("toolbar node");
    assert_eq!(toolbar.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        toolbar.component_ref.as_deref(),
        Some(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE)
    );
}

#[test]
fn ui_asset_editor_bootstrap_layout_self_hosts_palette_source_and_theme_regions() {
    let layout = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");

    for required_node in [
        "palette_panel",
        "source_panel",
        "stylesheet_panel",
        "preview_panel",
        "theme_tools_label",
        "theme_cascade_helper_label",
        "source_outline_label",
        "preview_graph_label",
        "preview_binding_schema_label",
        "command_log_label",
    ] {
        assert!(
            layout.contains_node(required_node),
            "bootstrap layout should include `{required_node}`"
        );
    }
}
