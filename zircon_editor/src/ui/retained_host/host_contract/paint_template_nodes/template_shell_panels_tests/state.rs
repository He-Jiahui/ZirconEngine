use crate::ui::layouts::common::model_rc;

use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{panel_node, pixel_at};

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
