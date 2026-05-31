use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::v2::UiV2AssetLoader;

fn workbench_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/windows/workbench_window.v2.ui.toml");
    fs::read_to_string(path).expect("workbench_window.v2.ui.toml should be readable")
}

#[test]
fn workbench_window_uses_reference_image_baseline() {
    let source = workbench_window_source();
    let document =
        UiV2AssetLoader::load_toml_str(&source).expect("workbench window v2 asset should parse");

    assert_eq!(document.asset.id, "editor.window.workbench");

    for marker in [
        "component = \"Image\"",
        "WorkbenchReferenceImage",
        "ui/editor/reference/workbench.png",
        "docs/ui-and-layout/workbench.png",
        "reference_width = 1672.0",
        "reference_height = 941.0",
        "aspect_ratio = 1.7768332",
    ] {
        assert!(source.contains(marker), "missing {marker}");
    }

    for control in [
        "WorkbenchWindowRoot",
        "WorkbenchReferenceFrame",
        "WorkbenchReferenceImage",
    ] {
        assert!(source.contains(control), "missing {control}");
    }
}
