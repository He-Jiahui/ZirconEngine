use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::template::UiAssetDocument;

fn editor_main_frame_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/host/editor_main_frame.ui.toml");
    fs::read_to_string(path).expect("editor_main_frame.ui.toml should be readable")
}

#[test]
fn editor_main_frame_asset_is_minimal_window_host() {
    let source = editor_main_frame_source();
    let document: UiAssetDocument =
        toml::from_str(&source).expect("editor main frame asset should parse");

    document
        .validate_tree_authority()
        .expect("editor main frame asset should validate");

    assert_eq!(document.asset.id, "editor.host.editor_main_frame");
    assert!(source.contains("slot_name = \"task_bar\""));
    assert!(source.contains("slot_name = \"window_tab_strip\""));
    assert!(source.contains("slot_name = \"active_window_host\""));

    for forbidden in [
        "ActivityRail",
        "DrawerShell",
        "DocumentHost",
        "WorkbenchShell",
        "BottomDrawer",
        "LeftDrawer",
        "RightDrawer",
    ] {
        assert!(
            !source.contains(forbidden),
            "editor main frame must not contain root-level {forbidden} business UI"
        );
    }
}
