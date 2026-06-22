use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, icon_node, pixel_at, positioned_icon_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn selected_toolbar_icon_button_paints_active_surface_and_glyph() {
    let bytes = paint_template_nodes_for_test(
        64,
        56,
        model_rc(vec![positioned_icon_node(
            "WorkbenchToolSelect",
            "zircon_editor_shell/toolbar/select.svg",
            true,
            8.0,
            8.0,
            48.0,
            40.0,
        )]),
    );

    assert_ne!(pixel_at(&bytes, 64, 16, 16), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 64, 22, 16, 22, 24) > 0);
}

#[test]
fn normal_toolbar_icon_button_keeps_outer_background_clean_and_draws_glyph() {
    let bytes = paint_template_nodes_for_test(
        48,
        48,
        model_rc(vec![icon_node(
            "WorkbenchToolbarMenu",
            "zircon_editor_shell/toolbar/menu.svg",
            false,
            34.0,
            34.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 48, 4, 4), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 48, 12, 12, 18, 18) > 0);
}

#[test]
fn selected_rail_icon_button_paints_large_active_surface() {
    let bytes = paint_template_nodes_for_test(
        64,
        64,
        model_rc(vec![positioned_icon_node(
            "WorkbenchRailScene",
            "zircon_editor_shell/activity/play.svg",
            true,
            8.0,
            8.0,
            48.0,
            48.0,
        )]),
    );

    assert_ne!(pixel_at(&bytes, 64, 16, 16), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 64, 25, 20, 18, 24) > 0);
}

#[test]
fn panel_danger_icon_button_paints_surface_and_error_glyph() {
    let bytes = paint_template_nodes_for_test(
        48,
        48,
        model_rc(vec![icon_node(
            "WorkbenchMiniDelete",
            "zircon_editor_shell/controls/delete.svg",
            false,
            36.0,
            36.0,
        )]),
    );

    assert_ne!(pixel_at(&bytes, 48, 8, 8), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 48, 14, 12, 20, 24) > 0);
}
