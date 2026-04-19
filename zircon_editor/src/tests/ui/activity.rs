use crate::{
    ui_asset_editor_window_descriptor, UiAssetEditorMode, UiAssetEditorRoute, UiAssetPreviewPreset,
};
use zircon_ui::{event_ui::UiNodePath, UiAssetKind};

#[test]
fn ui_asset_editor_window_descriptor_matches_shared_asset_editor_contract() {
    let descriptor = ui_asset_editor_window_descriptor();

    assert_eq!(descriptor.window_id, "editor.ui_asset");
    assert_eq!(descriptor.title, "UI Asset Editor");
    assert!(descriptor.multi_instance);
    assert!(descriptor.supports_document_tab);
    assert!(descriptor.supports_exclusive_page);
    assert!(descriptor.supports_floating_window);
    assert_eq!(
        descriptor.reflection_root,
        UiNodePath::new("editor/windows/editor.ui_asset")
    );
}

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
