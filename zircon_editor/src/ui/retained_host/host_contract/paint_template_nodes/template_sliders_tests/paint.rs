use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::{
    WORKBENCH_SLIDER_THUMB as SLIDER_THUMB, WORKBENCH_SLIDER_TRACK as SLIDER_TRACK,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, pixel_at, positioned_slider_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn workbench_slider_paints_track_fill_thumb_and_value() {
    let bytes = paint_template_nodes_for_test(
        220,
        48,
        model_rc(vec![positioned_slider_node(
            "WorkbenchInputSlider",
            0.75,
            8.0,
            8.0,
            184.0,
            30.0,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 220, 24, 23), PALETTE.accent);
    assert_eq!(pixel_at(&bytes, 220, 126, 23), SLIDER_TRACK);
    assert_eq!(pixel_at(&bytes, 220, 104, 23), SLIDER_THUMB);
    assert_ne!(pixel_at(&bytes, 220, 152, 23), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 220, 157, 16, 28, 16) > 0);
}

#[test]
fn hovered_workbench_slider_paints_thumb_halo() {
    let mut node = positioned_slider_node("WorkbenchInputSlider", 0.5, 8.0, 8.0, 160.0, 30.0);
    node.hovered = true;
    let bytes = paint_template_nodes_for_test(190, 48, model_rc(vec![node]));

    assert_ne!(pixel_at(&bytes, 190, 61, 15), [0, 0, 0, 255]);
}
