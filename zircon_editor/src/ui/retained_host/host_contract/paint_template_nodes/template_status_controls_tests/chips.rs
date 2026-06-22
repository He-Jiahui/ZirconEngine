use super::super::super::super::data::FrameRect;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::{
    select_workbench_status_chip_style, status_chip_text_color, status_control_offset_rect,
    PALETTE, STATUS_RIGHT_BORDER,
};
use super::support::{changed_pixel_count, pixel_at, status_chip_node};
use crate::ui::layouts::common::model_rc;

#[test]
fn status_chip_paints_pill_surface_and_down_chevron() {
    let bytes = paint_template_nodes_for_test(
        140,
        48,
        model_rc(vec![status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm")]),
    );

    assert_ne!(pixel_at(&bytes, 140, 20, 20), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 140, 60, 9), STATUS_RIGHT_BORDER);
    assert!(changed_pixel_count(&bytes, 140, 101, 18, 18, 14) > 0);
}

#[test]
fn status_chip_uses_declared_text_color_and_layout_offset() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.layout_offset_y = -2.0;
    node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(125, 137, 144);

    let rect = status_control_offset_rect(
        &node,
        &FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        },
    );

    assert!((rect.y - 7.0).abs() < 0.001);
    assert_eq!(status_chip_text_color(&node), [125, 137, 144, 255]);
}

#[test]
fn status_chip_uses_shared_painter_state_priority() {
    let mut node = status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm");
    node.hovered = true;
    node.selected = true;
    let hovered = select_workbench_status_chip_style(&node);
    assert_eq!(
        hovered.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Hovered
    );
    assert_eq!(hovered.background, PALETTE.surface_hover);

    node.pressed = true;
    let pressed = select_workbench_status_chip_style(&node);
    assert_eq!(
        pressed.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Pressed
    );
    assert_eq!(pressed.border, PALETTE.focus_ring);

    node.disabled = true;
    let disabled = select_workbench_status_chip_style(&node);
    assert_eq!(
        disabled.state,
        zircon_runtime_interface::ui::style::UiPainterResolvedState::Disabled
    );
    assert_eq!(disabled.background, PALETTE.surface_disabled);
}
