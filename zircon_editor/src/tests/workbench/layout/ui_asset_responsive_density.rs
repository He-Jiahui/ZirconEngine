const EDITOR_TOKENS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/theme/editor_tokens.zui");
const UI_ASSET_EDITOR_ASSET: &str =
    include_str!("../../../../assets/ui/editor/ui_asset_editor.zui");

#[test]
fn ui_asset_editor_uses_shared_density_constraints_for_chrome_columns() {
    for token in [
        "$editor.density.ui_asset.side.min_width",
        "$editor.density.ui_asset.side.preferred_width",
        "$editor.density.ui_asset.center.min_width",
        "$editor.density.ui_asset.center.preferred_width",
        "$editor.density.ui_asset.header_kind.min_width",
        "$editor.density.ui_asset.header_kind.preferred_width",
        "$editor.density.ui_asset.header_kind.max_width",
        "$editor.density.ui_asset.tool.min_width",
        "$editor.density.ui_asset.tool.preferred_width",
        "$editor.density.ui_asset.tool.max_width",
    ] {
        assert!(
            UI_ASSET_EDITOR_ASSET.contains(token),
            "UI Asset Editor chrome must resolve `{token}` through the density cascade"
        );
        assert!(
            EDITOR_TOKENS_ASSET.contains(&token[1..]),
            "editor tokens must name `{token}` for V2 resolution"
        );
    }

    for legacy_constraint in [
        "min = 128.0, preferred = 220.0",
        "min = 256.0, preferred = 420.0",
        "min = 64.0, preferred = 70.0, max = 84.0",
        "min = 56.0, preferred = 60.0, max = 68.0",
        "min = 60.0, preferred = 64.0, max = 72.0",
        "min = 64.0, preferred = 70.0, max = 80.0",
    ] {
        assert!(
            !UI_ASSET_EDITOR_ASSET.contains(legacy_constraint),
            "UI Asset Editor chrome must not retain the local constraint `{legacy_constraint}`"
        );
    }
}
