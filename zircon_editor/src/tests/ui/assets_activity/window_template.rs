use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::v2::UiV2AssetLoader;

fn asset_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/windows/asset_window.v2.ui.toml");
    fs::read_to_string(path).expect("asset_window.v2.ui.toml should be readable")
}

#[test]
fn asset_window_uses_activity_drawer_window_with_asset_browser_content() {
    let source = asset_window_source();
    let document =
        UiV2AssetLoader::load_toml_str(&source).expect("asset window v2 asset should parse");

    assert_eq!(document.asset.id, "editor.window.asset");
    assert!(source.contains("editor.host.activity_drawer_window#ActivityDrawerWindow"));
    assert!(source.contains("shell_preset = \"jetbrains_shell\""));
    assert!(source.contains("panel_preset = \"fyrox_panel\""));
    assert!(source.contains("window_model = \"unreal_window_model\""));

    for control in [
        "AssetWindowTreeActivity",
        "AssetWindowDetailsActivity",
        "AssetWindowPreviewActivity",
        "AssetWindowBrowserContent",
    ] {
        assert!(source.contains(control), "missing {control}");
    }
}
