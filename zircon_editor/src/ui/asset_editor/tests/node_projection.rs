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
