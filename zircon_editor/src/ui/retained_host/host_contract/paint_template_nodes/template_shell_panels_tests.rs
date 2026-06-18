use super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::super::style_selector::{
    WORKBENCH_CHROME_DRAWER_BG as DRAWER_BG, WORKBENCH_CHROME_PANEL_BG as PANEL_BG,
    WORKBENCH_CHROME_SOFT_SEPARATOR as SOFT_SEPARATOR, WORKBENCH_CHROME_STATUS_BG as STATUS_BG,
    WORKBENCH_CHROME_STRONG_SEPARATOR as STRONG_SEPARATOR, WORKBENCH_CHROME_TOPBAR_BG as TOPBAR_BG,
};
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_shell_panels_match_only_container_ids() {
    assert_eq!(
        shell_panel_kind(&panel_node(
            "WorkbenchWindowTopToolbar",
            0.0,
            0.0,
            120.0,
            40.0
        )),
        Some(ShellPanelKind::TopToolbar)
    );
    assert_eq!(
        shell_panel_kind(&panel_node(
            "WorkbenchInspectorPanel",
            0.0,
            0.0,
            120.0,
            40.0
        )),
        Some(ShellPanelKind::InspectorPanel)
    );
    assert_eq!(
        shell_panel_kind(&panel_node(
            "WorkbenchComponentInputs",
            0.0,
            0.0,
            120.0,
            40.0
        )),
        Some(ShellPanelKind::DrawerColumn)
    );
    assert_eq!(
        shell_panel_kind(&panel_node("WorkbenchViewportMode", 0.0, 0.0, 120.0, 40.0)),
        None
    );
}

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
fn shell_panel_chrome_selector_states_reach_native_paint() {
    let mut loading_status = panel_node("WorkbenchWindowStatusBar", 12.0, 8.0, 118.0, 26.0);
    loading_status.button_style.loading = true;
    loading_status.focused = true;
    loading_status.selected = true;

    let mut focused_inspector = panel_node("WorkbenchInspectorPanel", 152.0, 8.0, 72.0, 48.0);
    focused_inspector.focused = true;

    let bytes =
        paint_template_nodes_for_test(240, 72, model_rc(vec![loading_status, focused_inspector]));

    assert_eq!(pixel_at(&bytes, 240, 20, 8), PALETTE.border_disabled);
    assert_eq!(pixel_at(&bytes, 240, 20, 22), PALETTE.surface_disabled);
    assert_eq!(pixel_at(&bytes, 240, 152, 24), PALETTE.focus_ring);
    assert_eq!(pixel_at(&bytes, 240, 184, 24), PALETTE.surface_selected);
}

fn panel_node(
    control_id: &'static str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "VerticalGroup".into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
