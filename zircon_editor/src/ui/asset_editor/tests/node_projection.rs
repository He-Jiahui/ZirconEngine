use super::{
    UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_PATH, UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_PATH,
};

#[test]
fn projection_cache_loads_the_bootstrap_asset_contract() {
    assert_eq!(
        UI_ASSET_EDITOR_BOOTSTRAP_LAYOUT_ASSET_PATH,
        "/assets/ui/editor/ui_asset_editor.zui"
    );
    assert_eq!(
        UI_ASSET_EDITOR_BOOTSTRAP_STYLE_ASSET_PATH,
        "/assets/ui/editor/theme/editor_tokens.zui"
    );
}

#[test]
fn ui_source_adapter_preserves_bootstrap_namespace_and_rejects_invalid_suffixes() {
    assert_eq!(
        super::ui_source_asset_path("/assets/ui/editor/ui_asset_editor.zui").unwrap(),
        super::ui_source_asset_path("ui/editor/ui_asset_editor.zui").unwrap()
    );
    for source in [
        "/assets/",
        "/assets/../editor.zui",
        "/assets/./editor.zui",
        "/assets//editor.zui",
        "//assets/ui/editor.zui",
        "/ui/editor.zui",
        "C:/ui/editor.zui",
        r"/assets/ui\editor.zui",
    ] {
        assert!(
            matches!(
                super::ui_source_asset_path(source),
                Err(zircon_runtime_interface::ui::v2::UiV2AssetError::InvalidDocument { .. })
            ),
            "{source}"
        );
    }
}
