use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::template::UiAssetDocument;

fn workbench_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/windows/workbench_window.ui.toml");
    fs::read_to_string(path).expect("workbench_window.ui.toml should be readable")
}

#[test]
fn workbench_window_uses_activity_drawer_window_slots() {
    let source = workbench_window_source();
    let document: UiAssetDocument =
        toml::from_str(&source).expect("workbench window asset should parse");
    document
        .validate_tree_authority()
        .expect("workbench window asset should validate");

    assert_eq!(document.asset.id, "editor.window.workbench");
    assert!(source.contains("activity_drawer_window.ui.toml#ActivityDrawerWindow"));

    for mount in [
        "left_top_activity",
        "left_bottom_activity",
        "right_top_activity",
        "right_bottom_activity",
        "bottom_left_activity",
        "bottom_right_activity",
        "content",
    ] {
        assert!(source.contains(&format!("mount = \"{mount}\"")));
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
