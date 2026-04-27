use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::template::UiAssetDocument;

fn ui_layout_editor_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/windows/ui_layout_editor_window.ui.toml");
    fs::read_to_string(path).expect("ui_layout_editor_window.ui.toml should be readable")
}

#[test]
fn ui_layout_editor_window_uses_activity_drawer_window_with_editor_content() {
    let source = ui_layout_editor_window_source();
    let document: UiAssetDocument =
        toml::from_str(&source).expect("ui layout editor window asset should parse");
    document
        .validate_tree_authority()
        .expect("ui layout editor window asset should validate");

    assert_eq!(document.asset.id, "editor.window.ui_layout_editor");
    assert!(source.contains("activity_drawer_window.ui.toml#ActivityDrawerWindow"));
    assert!(source.contains("ui_asset_editor.ui.toml"));

    for control in [
        "UILayoutEditorPaletteActivity",
        "UILayoutEditorHierarchyActivity",
        "UILayoutEditorInspectorActivity",
        "UILayoutEditorStyleActivity",
        "UILayoutEditorDiagnosticsActivity",
        "UILayoutEditorContent",
    ] {
        assert!(source.contains(control), "missing {control}");
    }
}
