use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::{OUTLINED_BORDER, OUTLINED_SURFACE, PRIMARY_SURFACE};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, pixel_at, positioned_button_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn primary_workbench_button_paints_filled_surface_and_center_text() {
    let bytes = paint_template_nodes_for_test(
        152,
        48,
        model_rc(vec![positioned_button_node(
            "WorkbenchPrimaryButton",
            "Primary",
            "filled",
            12.0,
            8.0,
            120.0,
            34.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 152, 24, 24), PRIMARY_SURFACE);
    assert!(changed_pixel_count(&bytes, 152, 48, 16, 56, 18) > 0);
    assert_eq!(pixel_at(&bytes, 152, 140, 24), [0, 0, 0, 255]);
}

#[test]
fn outlined_workbench_button_paints_dark_surface_and_border() {
    let bytes = paint_template_nodes_for_test(
        152,
        48,
        model_rc(vec![positioned_button_node(
            "WorkbenchSecondaryButton",
            "Secondary",
            "outlined",
            12.0,
            8.0,
            120.0,
            34.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 152, 24, 24), OUTLINED_SURFACE);
    assert_eq!(pixel_at(&bytes, 152, 72, 8), OUTLINED_BORDER);
    assert!(changed_pixel_count(&bytes, 152, 42, 16, 70, 18) > 0);
}

#[test]
fn disabled_workbench_button_uses_disabled_surface_and_text() {
    let mut node = positioned_button_node(
        "WorkbenchDisabledButton",
        "Disabled",
        "outlined",
        12.0,
        8.0,
        120.0,
        34.0,
    );
    node.disabled = true;
    let bytes = paint_template_nodes_for_test(152, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 152, 24, 24), PALETTE.surface_disabled);
    assert_eq!(pixel_at(&bytes, 152, 72, 8), PALETTE.border_disabled);
    assert!(changed_pixel_count(&bytes, 152, 45, 16, 62, 18) > 0);
}

#[test]
fn dropdown_workbench_button_paints_trailing_chevron() {
    let bytes = paint_template_nodes_for_test(
        152,
        48,
        model_rc(vec![positioned_button_node(
            "WorkbenchDropdownButton",
            "Dropdown",
            "outlined",
            12.0,
            8.0,
            120.0,
            34.0,
        )]),
    );

    assert!(changed_pixel_count(&bytes, 152, 106, 18, 16, 12) > 0);
}
