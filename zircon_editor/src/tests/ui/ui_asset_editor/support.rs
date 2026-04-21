use crate::ui::asset_editor::{
    UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
    UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_ID, UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_BUTTON_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_SECTION_CARD_REFERENCE,
    UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOOLBAR_REFERENCE,
};
use zircon_runtime::ui::template::UiDocumentCompiler;
use zircon_runtime::ui::{layout::UiSize, template::UiAssetKind};

pub(super) const UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/ui_asset_editor.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/editor_widgets.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_ASSET_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/asset_browser.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_THEME_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/theme_browser.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_BINDING_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/binding_browser.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_LAYOUT_WORKBENCH_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/layout_workbench.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_PREVIEW_STATE_LAB_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/preview_state_lab.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_HUD_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/runtime_hud.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_DIALOG_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/pause_dialog.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_SETTINGS_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/settings_dialog.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_INVENTORY_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/inventory_dialog.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_QUEST_LOG_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/runtime/quest_log_dialog.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/theme/editor_base.ui.toml"
));

pub(super) fn register_bootstrap_imports(compiler: &mut UiDocumentCompiler) {
    let widget = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");

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
}

pub(super) fn hydrate_bootstrap_imports(session: &mut UiAssetEditorSession) {
    let widget = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = crate::tests::support::load_test_ui_asset(UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML)
        .expect("bootstrap style asset");

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
}

pub(super) fn open_design_session(asset_id: &str, source: &str) -> UiAssetEditorSession {
    UiAssetEditorSession::from_source(
        UiAssetEditorRoute::new(asset_id, UiAssetKind::Layout, UiAssetEditorMode::Design),
        source,
        UiSize::new(1280.0, 720.0),
    )
    .expect("design session")
}

pub(super) fn open_preview_session(asset_id: &str, source: &str) -> UiAssetEditorSession {
    UiAssetEditorSession::from_source(
        UiAssetEditorRoute::new(asset_id, UiAssetKind::Layout, UiAssetEditorMode::Preview),
        source,
        UiSize::new(1920.0, 1080.0),
    )
    .expect("preview session")
}
