use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;

use super::super::super::template_axis_value_field_style::{
    axis_field_background, axis_field_background_from_host, axis_field_border,
    axis_field_border_from_host, axis_field_border_width, axis_field_text_color,
    axis_field_text_color_from_host,
};
use super::support::axis_node;

#[test]
fn axis_value_field_uses_declared_value_color_when_present() {
    let mut node = axis_node("WorkbenchTransformPositionX", "128.4");
    node.value_color = Color::from_rgb_u8(146, 158, 164);

    assert_eq!(axis_field_text_color(&node), [146, 158, 164, 255]);
}

#[test]
fn focused_axis_value_field_keeps_normal_background() {
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
    node.focused = true;

    assert_eq!(axis_field_background(&node), PALETTE.surface_inset);
}

#[test]
fn hovered_axis_value_field_still_uses_hover_background() {
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
    node.hovered = true;

    assert_eq!(axis_field_background(&node), PALETTE.surface_hover);
}

#[test]
fn selected_axis_value_field_still_uses_hover_background() {
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
    node.selected = true;

    assert_eq!(axis_field_background(&node), PALETTE.surface_hover);
}

#[test]
fn selected_axis_value_field_uses_hover_border_without_focus_width() {
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
    node.selected = true;

    assert_eq!(axis_field_border(&node), PALETTE.separator_strong);
    assert_ne!(axis_field_border(&node), PALETTE.focus_ring);
    assert_eq!(axis_field_border_width(&node), 1.0);
}

#[test]
fn focused_axis_value_field_uses_focus_border_and_width() {
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
    node.focused = true;

    assert_eq!(axis_field_border(&node), PALETTE.focus_ring);
    assert_eq!(axis_field_border_width(&node), 1.5);
}

#[test]
fn axis_value_field_surface_and_border_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_inset = [10, 11, 12, 255];
    palette.surface_hover = [20, 21, 22, 255];
    palette.surface_selected = [30, 31, 32, 255];
    palette.surface_disabled = [40, 41, 42, 255];
    palette.border = [50, 51, 52, 255];
    palette.separator_strong = [60, 61, 62, 255];
    palette.border_disabled = [70, 71, 72, 255];
    palette.focus_ring = [80, 81, 82, 255];
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");

    assert_eq!(
        axis_field_background_from_host(&node, palette),
        [10, 11, 12, 255]
    );
    assert_eq!(
        axis_field_border_from_host(&node, palette),
        [50, 51, 52, 255]
    );

    node.hovered = true;
    assert_eq!(
        axis_field_background_from_host(&node, palette),
        [20, 21, 22, 255]
    );
    assert_eq!(
        axis_field_border_from_host(&node, palette),
        [60, 61, 62, 255]
    );

    node.hovered = false;
    node.pressed = true;
    assert_eq!(
        axis_field_background_from_host(&node, palette),
        [30, 31, 32, 255]
    );
    assert_eq!(
        axis_field_border_from_host(&node, palette),
        [80, 81, 82, 255]
    );

    node.pressed = false;
    node.disabled = true;
    assert_eq!(
        axis_field_background_from_host(&node, palette),
        [40, 41, 42, 255]
    );
    assert_eq!(
        axis_field_border_from_host(&node, palette),
        [70, 71, 72, 255]
    );
}

#[test]
fn axis_value_field_text_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.text = [10, 11, 12, 255];
    palette.text_disabled = [20, 21, 22, 255];
    palette.error = [30, 31, 32, 255];
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");

    assert_eq!(
        axis_field_text_color_from_host(&node, palette),
        [10, 11, 12, 255]
    );

    node.disabled = true;
    assert_eq!(
        axis_field_text_color_from_host(&node, palette),
        [20, 21, 22, 255]
    );

    node.disabled = false;
    node.validation_level = "error".into();
    assert_eq!(
        axis_field_text_color_from_host(&node, palette),
        [30, 31, 32, 255]
    );

    node.validation_level.clear();
    node.value_color = Color::from_rgb_u8(90, 91, 92);
    assert_eq!(
        axis_field_text_color_from_host(&node, palette),
        [90, 91, 92, 255]
    );
}

#[test]
fn pressed_axis_value_field_uses_focus_border_and_width() {
    let mut node = axis_node("WorkbenchTransformRotationY", "90 deg");
    node.pressed = true;

    assert_eq!(axis_field_border(&node), PALETTE.focus_ring);
    assert_eq!(axis_field_border_width(&node), 1.5);
}
