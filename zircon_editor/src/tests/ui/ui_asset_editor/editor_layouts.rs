use super::support::{
    hydrate_bootstrap_imports, open_design_session, register_bootstrap_imports,
    UI_ASSET_EDITOR_ASSET_BROWSER_TOML, UI_ASSET_EDITOR_BINDING_BROWSER_TOML,
    UI_ASSET_EDITOR_LAYOUT_WORKBENCH_TOML, UI_ASSET_EDITOR_PREVIEW_STATE_LAB_TOML,
    UI_ASSET_EDITOR_THEME_BROWSER_TOML,
};
use zircon_runtime::ui::template::UiDocumentCompiler;

fn assert_editor_layout_compiles_and_opens(
    source: &str,
    expected_asset_id: &str,
    route_id: &str,
    hierarchy_fragment: &str,
    canvas_label: &str,
) {
    let layout = crate::tests::support::load_test_ui_asset(source).expect("editor layout asset");
    let mut compiler = UiDocumentCompiler::default();
    register_bootstrap_imports(&mut compiler);

    let compiled = compiler.compile(&layout).expect("compile editor layout");
    assert_eq!(compiled.asset.id, expected_asset_id);

    let mut session = open_design_session(route_id, source);
    hydrate_bootstrap_imports(&mut session);

    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains(hierarchy_fragment)));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == canvas_label));
}

#[test]
fn ui_asset_editor_additional_editor_asset_browser_layout_compiles_and_opens() {
    assert_editor_layout_compiles_and_opens(
        UI_ASSET_EDITOR_ASSET_BROWSER_TOML,
        "editor.asset_browser",
        "res://ui/editor/asset_browser.ui.toml",
        "asset_browser_root [VerticalBox]",
        "AssetBrowserRoot",
    );
}

#[test]
fn ui_asset_editor_additional_editor_theme_browser_layout_compiles_and_opens() {
    assert_editor_layout_compiles_and_opens(
        UI_ASSET_EDITOR_THEME_BROWSER_TOML,
        "editor.theme_browser",
        "res://ui/editor/theme_browser.ui.toml",
        "theme_browser_root [VerticalBox]",
        "ThemeBrowserRoot",
    );
}

#[test]
fn ui_asset_editor_additional_editor_binding_browser_layout_compiles_and_opens() {
    assert_editor_layout_compiles_and_opens(
        UI_ASSET_EDITOR_BINDING_BROWSER_TOML,
        "editor.binding_browser",
        "res://ui/editor/binding_browser.ui.toml",
        "binding_browser_root [VerticalBox]",
        "BindingBrowserRoot",
    );
}

#[test]
fn ui_asset_editor_additional_editor_layout_workbench_asset_compiles_and_opens() {
    assert_editor_layout_compiles_and_opens(
        UI_ASSET_EDITOR_LAYOUT_WORKBENCH_TOML,
        "editor.layout_workbench",
        "res://ui/editor/layout_workbench.ui.toml",
        "layout_workbench_root [VerticalBox]",
        "LayoutWorkbenchRoot",
    );
}

#[test]
fn ui_asset_editor_additional_editor_preview_state_lab_asset_compiles_and_opens() {
    assert_editor_layout_compiles_and_opens(
        UI_ASSET_EDITOR_PREVIEW_STATE_LAB_TOML,
        "editor.preview_state_lab",
        "res://ui/editor/preview_state_lab.ui.toml",
        "preview_state_lab_root [VerticalBox]",
        "PreviewStateLabRoot",
    );
}
