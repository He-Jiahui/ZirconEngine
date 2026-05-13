use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
use zircon_runtime::ui::{template::UiDocumentCompiler, v2::UiV2AssetLoader};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

pub(super) const UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/editor/ui_asset_editor.v2.ui.toml"
));
const LEGACY_EDITOR_WIDGET_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tests/fixtures/ui_legacy/editor/editor_widgets.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_ASSET_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tests/fixtures/ui_legacy/editor/asset_browser.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_THEME_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tests/fixtures/ui_legacy/editor/theme_browser.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_BINDING_BROWSER_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tests/fixtures/ui_legacy/editor/binding_browser.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_LAYOUT_WORKBENCH_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tests/fixtures/ui_legacy/editor/layout_workbench.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_PREVIEW_STATE_LAB_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tests/fixtures/ui_legacy/editor/preview_state_lab.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_HUD_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/ui/runtime/fixtures/hud_overlay.v2.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_DIALOG_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/ui/runtime/fixtures/pause_menu.v2.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_SETTINGS_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/ui/runtime/fixtures/settings_dialog.v2.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_INVENTORY_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/ui/runtime/fixtures/inventory_list.v2.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_RUNTIME_QUEST_LOG_V2_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../zircon_runtime/assets/ui/runtime/fixtures/quest_log_dialog.v2.ui.toml"
));
pub(super) const UI_ASSET_EDITOR_BOOTSTRAP_STYLE_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/ui/theme/editor_material.v2.ui.toml"
));

const LEGACY_EDITOR_BASE_STYLE_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/tests/fixtures/ui_legacy/theme/editor_base.ui.toml"
));
const LEGACY_EDITOR_BASE_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.ui.toml";
const LEGACY_EDITOR_WIDGET_TOOLBAR_REFERENCE: &str =
    "res://ui/editor/editor_widgets.ui.toml#EditorToolbar";
const LEGACY_EDITOR_WIDGET_HEADER_SHELL_REFERENCE: &str =
    "res://ui/editor/editor_widgets.ui.toml#EditorHeaderShell";
const LEGACY_EDITOR_WIDGET_BUTTON_REFERENCE: &str =
    "res://ui/editor/editor_widgets.ui.toml#EditorToolbarButton";
const LEGACY_EDITOR_WIDGET_SECTION_CARD_REFERENCE: &str =
    "res://ui/editor/editor_widgets.ui.toml#EditorSectionCard";

pub(super) fn register_bootstrap_imports(compiler: &mut UiDocumentCompiler) {
    let widget = crate::tests::support::load_test_ui_asset(LEGACY_EDITOR_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = crate::tests::support::load_test_ui_asset(LEGACY_EDITOR_BASE_STYLE_TOML)
        .expect("bootstrap style asset");

    compiler
        .register_widget_import(LEGACY_EDITOR_WIDGET_TOOLBAR_REFERENCE, widget.clone())
        .expect("register bootstrap toolbar import");
    compiler
        .register_widget_import(LEGACY_EDITOR_WIDGET_HEADER_SHELL_REFERENCE, widget.clone())
        .expect("register bootstrap header shell import");
    compiler
        .register_widget_import(LEGACY_EDITOR_WIDGET_BUTTON_REFERENCE, widget.clone())
        .expect("register bootstrap widget import");
    compiler
        .register_widget_import(LEGACY_EDITOR_WIDGET_SECTION_CARD_REFERENCE, widget)
        .expect("register bootstrap section card import");
    compiler
        .register_style_import(LEGACY_EDITOR_BASE_STYLE_ASSET_ID, style)
        .expect("register bootstrap style import");
}

pub(super) fn hydrate_bootstrap_imports(session: &mut UiAssetEditorSession) {
    let widget = crate::tests::support::load_test_ui_asset(LEGACY_EDITOR_WIDGET_TOML)
        .expect("bootstrap widget asset");
    let style = crate::tests::support::load_test_ui_asset(LEGACY_EDITOR_BASE_STYLE_TOML)
        .expect("bootstrap style asset");

    session
        .register_widget_import(LEGACY_EDITOR_WIDGET_TOOLBAR_REFERENCE, widget.clone())
        .expect("hydrate bootstrap toolbar import");
    session
        .register_widget_import(LEGACY_EDITOR_WIDGET_HEADER_SHELL_REFERENCE, widget.clone())
        .expect("hydrate bootstrap header shell import");
    session
        .register_widget_import(LEGACY_EDITOR_WIDGET_BUTTON_REFERENCE, widget.clone())
        .expect("hydrate bootstrap widget import");
    session
        .register_widget_import(LEGACY_EDITOR_WIDGET_SECTION_CARD_REFERENCE, widget)
        .expect("hydrate bootstrap section card import");
    session
        .register_style_import(LEGACY_EDITOR_BASE_STYLE_ASSET_ID, style)
        .expect("hydrate bootstrap style import");
}

pub(super) fn open_design_session(asset_id: &str, source: &str) -> UiAssetEditorSession {
    if UiV2AssetLoader::load_toml_str(source).is_ok() {
        return UiAssetEditorSession::from_v2_source(
            UiAssetEditorRoute::new(asset_id, UiAssetKind::Layout, UiAssetEditorMode::Design),
            source,
            UiSize::new(1280.0, 720.0),
        )
        .expect("v2 design session");
    }

    UiAssetEditorSession::from_source(
        UiAssetEditorRoute::new(asset_id, UiAssetKind::Layout, UiAssetEditorMode::Design),
        source,
        UiSize::new(1280.0, 720.0),
    )
    .expect("design session")
}

pub(super) fn open_v2_preview_session(asset_id: &str, source: &str) -> UiAssetEditorSession {
    UiAssetEditorSession::from_v2_source(
        UiAssetEditorRoute::new(asset_id, UiAssetKind::Layout, UiAssetEditorMode::Preview),
        source,
        UiSize::new(1920.0, 1080.0),
    )
    .expect("preview session")
}
