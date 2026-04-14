use std::fs;
use std::path::PathBuf;

fn shell_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench.slint");
    fs::read_to_string(path).expect("workbench.slint should be readable")
}

fn block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker `{marker}` in workbench.slint"));
    let end = (start + 5000).min(source.len());
    &source[start..end]
}

#[test]
fn shell_regions_bind_absolute_anchors_from_solver_frames() {
    let source = shell_source();

    let center_band = block_after(&source, "main_content_zone := Rectangle {");
    assert!(center_band.contains("x: root.center_band_frame.x * 1px;"));
    assert!(center_band.contains("y: root.center_band_frame.y * 1px;"));
    assert!(center_band.contains("width: root.center_band_frame.width * 1px;"));
    assert!(center_band.contains("height: root.center_band_frame.height * 1px;"));

    let right_region = block_after(&source, "right_stack_zone := Rectangle {");
    assert!(right_region.contains("x: root.right_region_frame.x * 1px;"));
    assert!(right_region.contains("y: root.right_region_frame.y * 1px;"));
    assert!(right_region.contains("width: root.right_region_frame.width * 1px;"));
    assert!(right_region.contains("height: root.right_region_frame.height * 1px;"));

    let bottom_region = block_after(&source, "bottom_panel_zone := Rectangle {");
    assert!(bottom_region.contains("x: root.bottom_region_frame.x * 1px;"));
    assert!(bottom_region.contains("y: root.bottom_region_frame.y * 1px;"));
    assert!(bottom_region.contains("width: root.bottom_region_frame.width * 1px;"));
    assert!(bottom_region.contains("height: root.bottom_region_frame.height * 1px;"));

    let status_bar = block_after(&source, "status_bar_zone := Rectangle {");
    assert!(status_bar.contains("x: root.status_bar_frame.x * 1px;"));
    assert!(status_bar.contains("y: root.status_bar_frame.y * 1px;"));
    assert!(status_bar.contains("width: root.status_bar_frame.width * 1px;"));
    assert!(status_bar.contains("height: root.status_bar_frame.height * 1px;"));
}

#[test]
fn workbench_shell_declares_native_resize_and_maximize_bounds() {
    let source = shell_source();
    let shell_block = block_after(&source, "export component WorkbenchShell inherits Window {");

    assert!(shell_block.contains("no-frame: false;"));
    assert!(shell_block.contains("resize-border-width: 8px;"));
    assert!(shell_block.contains("max-width:"));
    assert!(shell_block.contains("max-height:"));
}

#[test]
fn shell_drag_targets_allow_empty_tool_regions() {
    let source = shell_source();

    assert!(source.contains(
        "callback drop_tab(tab_id: string, target_group: string, pointer_x: float, pointer_y: float);"
    ));

    let drag_target = block_after(&source, "property <string> drag_target_group:");
    assert!(
        drag_target.contains("root.prefer_left_drop_over_bottom ? \"left\" :")
    );
    assert!(
        drag_target.contains("root.prefer_right_drop_over_bottom ? \"right\" :")
    );
    assert!(
        drag_target.contains("root.pointer_in_bottom_drop ? \"bottom\" :")
    );
    assert!(source.contains("property <bool> pointer_in_right_drop:"));
    assert!(source.contains("property <float> right_edge_distance_px:"));
    assert!(source.contains("property <bool> prefer_right_drop_over_bottom:"));
    assert!(!drag_target.contains("root.bottom_tabs.length > 0 || root.bottom_panel_height > 0px"));

    let drag_overlay = block_after(&source, "if root.drag_active: Rectangle {");
    assert!(drag_overlay.contains("if root.left_drop_enabled: Rectangle {"));
    assert!(drag_overlay.contains("if root.right_drop_enabled: Rectangle {"));
    assert!(drag_overlay.contains("x: parent.width - root.right_drop_width + 8px;"));
    assert!(drag_overlay.contains("if root.bottom_drop_enabled: Rectangle {"));

    assert!(drag_overlay.contains("if (root.drag_tab_id != \"\" && root.drag_target_group != \"\") {"));
    assert!(drag_overlay.contains("root.drop_tab("));
    assert!(drag_overlay.contains("root.drag_tab_id,"));
    assert!(drag_overlay.contains("root.drag_target_group,"));
    assert!(drag_overlay.contains("root.drag_pointer_x,"));
    assert!(drag_overlay.contains("root.drag_pointer_y,"));
    assert!(!drag_overlay.contains("root.drag_target_group != root.drag_source_group"));
}
