use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetPreviewPreset};
use zircon_runtime_interface::ui::template::UiAssetKind;

#[test]
fn ui_asset_editor_route_captures_asset_kind_and_editor_mode() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/editor/ui_asset_editor.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );

    assert_eq!(route.asset_id, "asset://ui/editor/ui_asset_editor.ui.toml");
    assert_eq!(route.asset_kind, UiAssetKind::Layout);
    assert_eq!(route.mode, UiAssetEditorMode::Split);
    assert_eq!(route.preview_preset, UiAssetPreviewPreset::EditorDocked);
    assert_eq!(route.window_id(), "editor.ui_asset");
}
