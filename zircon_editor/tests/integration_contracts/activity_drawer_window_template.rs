use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::v2::UiV2AssetLoader;

fn activity_drawer_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/host/activity_drawer_window.v2.ui.toml");
    fs::read_to_string(path).expect("activity_drawer_window.v2.ui.toml should be readable")
}

#[test]
fn activity_drawer_window_declares_neutral_drawer_slots() {
    let source = activity_drawer_window_source();
    let document = UiV2AssetLoader::load_toml_str(&source)
        .expect("activity drawer window v2 asset should parse");

    let component = document
        .components
        .get("ActivityDrawerWindow")
        .expect("ActivityDrawerWindow component should exist");

    for slot in [
        "left_top_activity",
        "left_bottom_activity",
        "right_top_activity",
        "right_bottom_activity",
        "bottom_left_activity",
        "bottom_right_activity",
        "content",
    ] {
        assert!(
            component.slots.contains_key(slot),
            "ActivityDrawerWindow should expose {slot} slot"
        );
        assert!(
            source.contains(&format!("name = \"{slot}\"")),
            "ActivityDrawerWindow tree should include {slot} slot node"
        );
    }

    for forbidden in [
        "Workbench",
        "AssetBrowser",
        "UiAssetEditor",
        "HierarchyToggle",
        "InspectorToggle",
        "ConsoleToggle",
    ] {
        assert!(
            !source.contains(forbidden),
            "ActivityDrawerWindow must stay business-neutral and not contain {forbidden}"
        );
    }
}
