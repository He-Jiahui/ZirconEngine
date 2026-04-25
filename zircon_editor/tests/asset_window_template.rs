use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::template::UiAssetDocument;

fn asset_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/windows/asset_window.ui.toml");
    fs::read_to_string(path).expect("asset_window.ui.toml should be readable")
}

#[test]
fn asset_window_uses_activity_drawer_window_with_asset_browser_content() {
    let source = asset_window_source();
    let document: UiAssetDocument =
        toml::from_str(&source).expect("asset window asset should parse");
    document
        .validate_tree_authority()
        .expect("asset window asset should validate");

    assert_eq!(document.asset.id, "editor.window.asset");
    assert!(source.contains("activity_drawer_window.ui.toml#ActivityDrawerWindow"));
    assert!(source.contains("asset_browser.ui.toml"));

    for control in [
        "AssetWindowTreeActivity",
        "AssetWindowDetailsActivity",
        "AssetWindowPreviewActivity",
        "AssetWindowBrowserContent",
    ] {
        assert!(source.contains(control), "missing {control}");
    }
}
