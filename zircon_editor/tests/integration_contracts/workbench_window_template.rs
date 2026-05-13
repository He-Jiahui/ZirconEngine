use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::v2::UiV2AssetLoader;

fn workbench_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/windows/workbench_window.v2.ui.toml");
    fs::read_to_string(path).expect("workbench_window.v2.ui.toml should be readable")
}

#[test]
fn workbench_window_uses_activity_drawer_window_slots() {
    let source = workbench_window_source();
    let document =
        UiV2AssetLoader::load_toml_str(&source).expect("workbench window v2 asset should parse");

    assert_eq!(document.asset.id, "editor.window.workbench");
    assert!(source.contains("editor.host.activity_drawer_window#ActivityDrawerWindow"));

    for mount in [
        "left_top_activity",
        "left_bottom_activity",
        "right_top_activity",
        "right_bottom_activity",
        "bottom_left_activity",
        "bottom_right_activity",
        "content",
    ] {
        assert!(source.contains(&format!("name = \"{mount}\"")));
    }

    for control in [
        "WorkbenchHierarchyActivity",
        "WorkbenchAssetsActivity",
        "WorkbenchInspectorActivity",
        "WorkbenchConsoleActivity",
        "WorkbenchDocumentContent",
    ] {
        assert!(source.contains(control), "missing {control}");
    }
}
