use serde_json::json;

use crate::{
    UiAssetEditorMode, UiAssetEditorReflectionModel, UiAssetEditorRoute, UiAssetEditorSession,
    UiDesignerSelectionModel, UiMatchedStyleRuleReflection, UiStyleInspectorReflectionModel,
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_DOCUMENT_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_ASSET_ID,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
};
use zircon_runtime::ui::template::UiDocumentCompiler;
use zircon_runtime::ui::template::UiNodeDefinitionKind;
use zircon_runtime::ui::{layout::UiSize, template::UiAssetKind, template::UiAssetLoader};

const UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/ui_asset_editor.ui.toml"
));
const UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/editor_widgets.ui.toml"
));
const UI_ASSET_EDITOR_ASSET_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/asset_browser.ui.toml"
));
const UI_ASSET_EDITOR_THEME_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/theme_browser.ui.toml"
));
const UI_ASSET_EDITOR_BINDING_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/binding_browser.ui.toml"
));
const UI_ASSET_EDITOR_LAYOUT_WORKBENCH_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/layout_workbench.ui.toml"
));
const UI_ASSET_EDITOR_PREVIEW_STATE_LAB_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/preview_state_lab.ui.toml"
));
const UI_ASSET_EDITOR_RUNTIME_HUD_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/runtime_hud.ui.toml"
));
const UI_ASSET_EDITOR_RUNTIME_DIALOG_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/pause_dialog.ui.toml"
));
const UI_ASSET_EDITOR_RUNTIME_SETTINGS_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/settings_dialog.ui.toml"
));
const UI_ASSET_EDITOR_RUNTIME_INVENTORY_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/inventory_dialog.ui.toml"
));
const UI_ASSET_EDITOR_RUNTIME_QUEST_LOG_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/quest_log_dialog.ui.toml"
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
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            widget.clone(),
        )
        .expect("register bootstrap toolbar import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            widget.clone(),
        )
        .expect("register bootstrap widget import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            widget,
        )
        .expect("register bootstrap section card import");
    compiler
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style)
        .expect("register bootstrap style import");

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
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            widget.clone(),
        )
        .expect("hydrate bootstrap toolbar import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            widget.clone(),
        )
        .expect("hydrate bootstrap widget import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            widget,
        )
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
    let layout = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
        .expect("bootstrap layout asset");

    assert!(layout
        .imports
        .widgets
        .iter()
        .any(|reference| reference == UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE));
    let toolbar = layout.nodes.get("toolbar").expect("toolbar node");
    assert_eq!(toolbar.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(
        toolbar.component_ref.as_deref(),
        Some(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE)
    );
}

#[test]
fn ui_asset_editor_additional_editor_asset_browser_layout_compiles_and_opens() {
    let layout = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_ASSET_BROWSER_TOML)
        .expect("asset browser layout asset");
    let widget = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            widget.clone(),
        )
        .expect("register toolbar import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            widget.clone(),
        )
        .expect("register button import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            widget,
        )
        .expect("register section card import");
    compiler
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style.clone())
        .expect("register style import");

    let compiled = compiler
        .compile(&layout)
        .expect("compile asset browser layout");
    assert_eq!(compiled.asset.id, "editor.asset_browser");

    let route = UiAssetEditorRoute::new(
        "res://ui/editor/asset_browser.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_ASSET_BROWSER_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("asset browser session");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate toolbar import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate button import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate section card import");
    session
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style)
        .expect("hydrate style import");

    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("asset_browser_root [VerticalBox]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "AssetBrowserRoot"));
}

#[test]
fn ui_asset_editor_additional_editor_theme_browser_layout_compiles_and_opens() {
    let layout = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_THEME_BROWSER_TOML)
        .expect("theme browser layout asset");
    let widget = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            widget.clone(),
        )
        .expect("register toolbar import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            widget.clone(),
        )
        .expect("register button import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            widget,
        )
        .expect("register section card import");
    compiler
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style.clone())
        .expect("register style import");

    let compiled = compiler
        .compile(&layout)
        .expect("compile theme browser layout");
    assert_eq!(compiled.asset.id, "editor.theme_browser");

    let route = UiAssetEditorRoute::new(
        "res://ui/editor/theme_browser.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_THEME_BROWSER_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("theme browser session");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate toolbar import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate button import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate section card import");
    session
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style)
        .expect("hydrate style import");

    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("theme_browser_root [VerticalBox]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "ThemeBrowserRoot"));
}

#[test]
fn ui_asset_editor_additional_editor_binding_browser_layout_compiles_and_opens() {
    let layout = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BINDING_BROWSER_TOML)
        .expect("binding browser layout asset");
    let widget = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            widget.clone(),
        )
        .expect("register toolbar import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            widget.clone(),
        )
        .expect("register button import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            widget,
        )
        .expect("register section card import");
    compiler
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style.clone())
        .expect("register style import");

    let compiled = compiler
        .compile(&layout)
        .expect("compile binding browser layout");
    assert_eq!(compiled.asset.id, "editor.binding_browser");

    let route = UiAssetEditorRoute::new(
        "res://ui/editor/binding_browser.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_BINDING_BROWSER_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("binding browser session");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate toolbar import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate button import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate section card import");
    session
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style)
        .expect("hydrate style import");

    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("binding_browser_root [VerticalBox]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "BindingBrowserRoot"));
}

#[test]
fn ui_asset_editor_runtime_hud_asset_opens_as_shared_runtime_preview_session() {
    let route = UiAssetEditorRoute::new(
        "res://ui/runtime/runtime_hud.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_RUNTIME_HUD_TOML,
        UiSize::new(1920.0, 1080.0),
    )
    .expect("runtime hud session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("hud_root [VerticalBox]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "RuntimeHudRoot"));
}

#[test]
fn ui_asset_editor_runtime_pause_dialog_asset_opens_as_shared_runtime_preview_session() {
    let route = UiAssetEditorRoute::new(
        "res://ui/runtime/pause_dialog.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_RUNTIME_DIALOG_TOML,
        UiSize::new(1920.0, 1080.0),
    )
    .expect("runtime pause dialog session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("pause_dialog_root [Overlay]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "PauseDialogRoot"));
}

#[test]
fn ui_asset_editor_runtime_settings_dialog_asset_opens_as_shared_runtime_preview_session() {
    let route = UiAssetEditorRoute::new(
        "res://ui/runtime/settings_dialog.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_RUNTIME_SETTINGS_TOML,
        UiSize::new(1920.0, 1080.0),
    )
    .expect("runtime settings dialog session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("settings_dialog_root [Overlay]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "SettingsDialogRoot"));
}

#[test]
fn ui_asset_editor_bootstrap_layout_self_hosts_palette_source_and_theme_regions() {
    let layout = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML)
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
            layout.nodes.contains_key(required_node),
            "bootstrap layout should include `{required_node}`"
        );
    }
}

#[test]
fn ui_asset_editor_additional_editor_layout_workbench_asset_compiles_and_opens() {
    let layout = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_LAYOUT_WORKBENCH_TOML)
        .expect("layout workbench asset");
    let widget = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            widget.clone(),
        )
        .expect("register toolbar import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            widget.clone(),
        )
        .expect("register button import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            widget,
        )
        .expect("register section card import");
    compiler
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style.clone())
        .expect("register style import");

    let compiled = compiler
        .compile(&layout)
        .expect("compile layout workbench asset");
    assert_eq!(compiled.asset.id, "editor.layout_workbench");

    let route = UiAssetEditorRoute::new(
        "res://ui/editor/layout_workbench.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_LAYOUT_WORKBENCH_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("layout workbench session");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate toolbar import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate button import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate section card import");
    session
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style)
        .expect("hydrate style import");

    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("layout_workbench_root [VerticalBox]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "LayoutWorkbenchRoot"));
}

#[test]
fn ui_asset_editor_runtime_inventory_dialog_asset_opens_as_shared_runtime_preview_session() {
    let route = UiAssetEditorRoute::new(
        "res://ui/runtime/inventory_dialog.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_RUNTIME_INVENTORY_TOML,
        UiSize::new(1920.0, 1080.0),
    )
    .expect("runtime inventory dialog session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("inventory_dialog_root [Overlay]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "InventoryDialogRoot"));
}

#[test]
fn ui_asset_editor_additional_editor_preview_state_lab_asset_compiles_and_opens() {
    let layout = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_PREVIEW_STATE_LAB_TOML)
        .expect("preview state lab asset");
    let widget = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            widget.clone(),
        )
        .expect("register toolbar import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            widget.clone(),
        )
        .expect("register button import");
    compiler
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            widget,
        )
        .expect("register section card import");
    compiler
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style.clone())
        .expect("register style import");

    let compiled = compiler
        .compile(&layout)
        .expect("compile preview state lab asset");
    assert_eq!(compiled.asset.id, "editor.preview_state_lab");

    let route = UiAssetEditorRoute::new(
        "res://ui/editor/preview_state_lab.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_PREVIEW_STATE_LAB_TOML,
        UiSize::new(1280.0, 720.0),
    )
    .expect("preview state lab session");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate toolbar import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate button import");
    session
        .register_widget_import(
            UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
            UiAssetLoader::load_toml_str(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
                .expect("widget rehydrate"),
        )
        .expect("hydrate section card import");
    session
        .register_style_import(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, style)
        .expect("hydrate style import");

    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("preview_state_lab_root [VerticalBox]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "PreviewStateLabRoot"));
}

#[test]
fn ui_asset_editor_runtime_quest_log_dialog_asset_opens_as_shared_runtime_preview_session() {
    let route = UiAssetEditorRoute::new(
        "res://ui/runtime/quest_log_dialog.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let session = UiAssetEditorSession::from_source(
        route,
        UI_ASSET_EDITOR_RUNTIME_QUEST_LOG_TOML,
        UiSize::new(1920.0, 1080.0),
    )
    .expect("runtime quest log dialog session");

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains("quest_log_dialog_root [Overlay]")));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == "QuestLogDialogRoot"));
}
