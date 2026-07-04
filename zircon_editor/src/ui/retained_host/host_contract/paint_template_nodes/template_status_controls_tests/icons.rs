use super::super::super::super::data::FrameRect;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::{
    select_workbench_status_icon_button_style, status_control_offset_rect,
    status_icon_button_glyph_rect, PALETTE,
};
use super::support::{changed_pixel_count, pixel_at, status_icon_node};
use crate::ui::layouts::common::model_rc;

const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

#[test]
fn status_icon_button_paints_target_glyph_without_button_surface() {
    let bytes = paint_template_nodes_for_test(
        48,
        42,
        model_rc(vec![status_icon_node("WorkbenchStatusTarget")]),
    );

    assert_eq!(pixel_at(&bytes, 48, 8, 8), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 48, 24, 6), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 48, 14, 11, 20, 20) > 0);
}

#[test]
fn status_icon_button_uses_declared_layout_offset() {
    let mut node = status_icon_node("WorkbenchStatusTarget");
    node.layout_offset_y = -2.0;

    let rect = status_control_offset_rect(
        &node,
        &FrameRect {
            x: 6.0,
            y: 6.0,
            width: 34.0,
            height: 30.0,
        },
    );

    assert!((rect.y - 4.0).abs() < 0.001);
}

#[test]
fn status_icon_button_glyph_rect_uses_shared_status_metrics() {
    let rect = status_icon_button_glyph_rect(&FrameRect {
        x: 6.0,
        y: 6.0,
        width: 34.0,
        height: 30.0,
    });

    assert!((rect.x - 15.0).abs() < 0.001);
    assert!((rect.y - 13.0).abs() < 0.001);
    assert!((rect.width - 16.0).abs() < 0.001);
    assert!((rect.height - 16.0).abs() < 0.001);
}

#[test]
fn status_icon_button_uses_shared_icon_button_state_priority() {
    let mut node = status_icon_node("WorkbenchStatusTarget");
    node.checked = true;
    let checked = select_workbench_status_icon_button_style(&node);
    assert_eq!(
        checked.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Checked
    );
    assert_eq!(checked.glyph, PALETTE.focus_ring);

    node.hovered = true;
    let hovered = select_workbench_status_icon_button_style(&node);
    assert_eq!(
        hovered.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Hovered
    );

    node.pressed = true;
    let pressed = select_workbench_status_icon_button_style(&node);
    assert_eq!(
        pressed.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
    );
    assert_eq!(pressed.background, PALETTE.surface_pressed);
}

#[test]
fn status_icon_button_normal_state_is_flat_transparent() {
    let node = status_icon_node("WorkbenchStatusTarget");

    let normal = select_workbench_status_icon_button_style(&node);

    assert_eq!(normal.background, TRANSPARENT);
    assert_eq!(normal.border, TRANSPARENT);
    assert_eq!(normal.glyph, PALETTE.text_disabled);
}
