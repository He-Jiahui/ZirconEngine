use crate::ui::layouts::common::model_rc;

use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::style::{CHIP_BORDER, CHIP_SURFACE};
use super::support::{changed_pixel_count, chip_node, pixel_at};

#[test]
fn viewport_chip_paints_surface_border_text_and_chevron() {
    let bytes = paint_template_nodes_for_test(
        150,
        48,
        model_rc(vec![chip_node("WorkbenchViewportMode", "Perspective")]),
    );

    assert_eq!(pixel_at(&bytes, 150, 110, 24), CHIP_SURFACE);
    assert_eq!(pixel_at(&bytes, 150, 54, 8), CHIP_BORDER);
    assert!(changed_pixel_count(&bytes, 150, 22, 16, 62, 18) > 0);
    assert!(changed_pixel_count(&bytes, 150, 102, 15, 18, 18) > 0);
}

#[test]
fn focused_chip_uses_focus_border() {
    let mut node = chip_node("WorkbenchViewportAngle", "10 deg");
    node.focused = true;
    let bytes = paint_template_nodes_for_test(120, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 120, 54, 8), PALETTE.focus_ring);
}
