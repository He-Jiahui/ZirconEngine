use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::super::style_selector::{
    WORKBENCH_CHROME_DRAWER_BG as DRAWER_BG, WORKBENCH_CHROME_PANEL_BG as PANEL_BG,
    WORKBENCH_CHROME_SOFT_SEPARATOR as SOFT_SEPARATOR, WORKBENCH_CHROME_STATUS_BG as STATUS_BG,
    WORKBENCH_CHROME_STRONG_SEPARATOR as STRONG_SEPARATOR, WORKBENCH_CHROME_TOPBAR_BG as TOPBAR_BG,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{panel_node, pixel_at};

#[test]
fn top_toolbar_paints_surface_and_bottom_separator() {
    let bytes = paint_template_nodes_for_test(
        150,
        64,
        model_rc(vec![panel_node(
            "WorkbenchWindowTopToolbar",
            8.0,
            6.0,
            128.0,
            40.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 150, 24, 18), TOPBAR_BG);
    assert_eq!(pixel_at(&bytes, 150, 24, 45), STRONG_SEPARATOR);
}

#[test]
fn side_panels_paint_directional_separators() {
    let bytes = paint_template_nodes_for_test(
        240,
        90,
        model_rc(vec![
            panel_node("WorkbenchSceneTreePanel", 8.0, 10.0, 86.0, 56.0),
            panel_node("WorkbenchInspectorPanel", 130.0, 10.0, 92.0, 56.0),
        ]),
    );

    assert_eq!(pixel_at(&bytes, 240, 30, 32), PANEL_BG);
    assert_eq!(pixel_at(&bytes, 240, 93, 32), STRONG_SEPARATOR);
    assert_eq!(pixel_at(&bytes, 240, 130, 32), STRONG_SEPARATOR);
    assert_eq!(pixel_at(&bytes, 240, 170, 32), PANEL_BG);
}

#[test]
fn drawer_and_status_bar_paint_top_separators() {
    let bytes = paint_template_nodes_for_test(
        180,
        90,
        model_rc(vec![
            panel_node("WorkbenchComponentDrawer", 10.0, 12.0, 140.0, 34.0),
            panel_node("WorkbenchWindowStatusBar", 10.0, 56.0, 140.0, 28.0),
        ]),
    );

    assert_eq!(pixel_at(&bytes, 180, 20, 12), STRONG_SEPARATOR);
    assert_eq!(pixel_at(&bytes, 180, 20, 24), DRAWER_BG);
    assert_eq!(pixel_at(&bytes, 180, 20, 56), STRONG_SEPARATOR);
    assert_eq!(pixel_at(&bytes, 180, 20, 68), STATUS_BG);
}

#[test]
fn drawer_column_paints_gap_separator_without_surface_fill() {
    let bytes = paint_template_nodes_for_test(
        160,
        70,
        model_rc(vec![panel_node(
            "WorkbenchComponentInputs",
            72.0,
            10.0,
            70.0,
            42.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 160, 66, 24), SOFT_SEPARATOR);
    assert_eq!(pixel_at(&bytes, 160, 96, 24), [0, 0, 0, 255]);
}

#[test]
fn module_content_panel_paints_rounded_bordered_surface() {
    let bytes = paint_template_nodes_for_test(
        150,
        90,
        model_rc(vec![panel_node(
            "WorkbenchAssetsLeftPanel",
            12.0,
            10.0,
            96.0,
            54.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 150, 30, 30), PANEL_BG);
    assert_eq!(pixel_at(&bytes, 150, 24, 10), PALETTE.border);
    assert_eq!(pixel_at(&bytes, 150, 12, 24), PALETTE.border);
    assert_eq!(pixel_at(&bytes, 150, 12, 10), [0, 0, 0, 255]);
}
