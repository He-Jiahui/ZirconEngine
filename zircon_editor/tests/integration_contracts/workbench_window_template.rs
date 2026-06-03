use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::v2::UiV2AssetLoader;

fn workbench_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/windows/workbench_window.v2.ui.toml");
    fs::read_to_string(path).expect("workbench_window.v2.ui.toml should be readable")
}

#[test]
fn workbench_window_uses_componentized_workbench_layout() {
    let source = workbench_window_source();
    let document =
        UiV2AssetLoader::load_toml_str(&source).expect("workbench window v2 asset should parse");

    assert_eq!(document.asset.id, "editor.window.workbench");

    for marker in [
        "editor_workbench_strict.v2.ui.toml",
        "res://ui/editor/components/workbench_component_drawer.zui#WorkbenchComponentDrawer",
        "res://ui/editor/components/workbench_main_band.zui#WorkbenchMainBand",
        "res://ui/editor/components/workbench_status_bar.zui#WorkbenchStatusBar",
        "res://ui/editor/components/workbench_top_toolbar.zui#WorkbenchTopToolbar",
        "component = \"WorkbenchTopToolbar\"",
        "component = \"WorkbenchMainBand\"",
        "component = \"WorkbenchComponentDrawer\"",
        "component = \"WorkbenchStatusBar\"",
    ] {
        assert!(source.contains(marker), "missing {marker}");
    }

    for forbidden in [
        "WorkbenchReferenceFrame",
        "WorkbenchReferenceImage",
        "ui/editor/reference/workbench.png",
        "docs/ui-and-layout/workbench.png",
        "component = \"IconButton\"",
        "component = \"Button\"",
        "component = \"Dropdown\"",
        "component = \"Checkbox\"",
        "component = \"Radio\"",
        "component = \"Toggle\"",
        "component = \"RangeField\"",
        "component = \"TreeRow\"",
        "component = \"ListRow\"",
        "component = \"ContextActionMenu\"",
        "res://ui/editor/components/workbench_button.zui#WorkbenchButton",
        "res://ui/editor/components/workbench_checkbox.zui#WorkbenchCheckbox",
        "res://ui/editor/components/workbench_dropdown.zui#WorkbenchDropdown",
        "res://ui/editor/components/workbench_field.zui#WorkbenchField",
        "res://ui/editor/components/workbench_icon_button.zui#WorkbenchIconButton",
        "res://ui/editor/components/workbench_popup_menu.zui#WorkbenchPopupMenu",
        "res://ui/editor/components/workbench_segmented_control.zui#WorkbenchSegmentedControl",
        "res://ui/editor/components/workbench_slider.zui#WorkbenchSlider",
        "res://ui/editor/components/workbench_table_row.zui#WorkbenchTableRow",
        "res://ui/editor/components/workbench_tree_row.zui#WorkbenchTreeRow",
    ] {
        assert!(
            !source.contains(forbidden),
            "workbench window must stay componentized instead of rendering `{forbidden}`"
        );
    }
}
