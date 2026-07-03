use super::super::super::style_selector::{
    WORKBENCH_DROPDOWN_BORDER as DROPDOWN_BORDER,
    WORKBENCH_DROPDOWN_FOCUS_BORDER as DROPDOWN_FOCUS_BORDER,
    WORKBENCH_DROPDOWN_SURFACE as DROPDOWN_SURFACE,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::push_dropdown_commands;
use super::support::{changed_pixel_count, dropdown_node, option, pixel_at, scaled_test_color};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn closed_workbench_dropdown_paints_surface_border_text_and_chevron() {
    let bytes = paint_template_nodes_for_test(140, 48, model_rc(vec![dropdown_node(false)]));

    assert_eq!(pixel_at(&bytes, 140, 88, 24), DROPDOWN_SURFACE);
    assert_eq!(pixel_at(&bytes, 140, 54, 8), DROPDOWN_BORDER);
    assert!(changed_pixel_count(&bytes, 140, 22, 16, 50, 18) > 0);
    assert!(changed_pixel_count(&bytes, 140, 96, 15, 18, 18) > 0);
}

#[test]
fn closed_workbench_dropdown_chevron_prefers_shell_asset_pixels() {
    let node = dropdown_node(false);
    let rect = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 104.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_dropdown_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));

    let icon_commands = commands
        .iter()
        .filter(|command| command.image_pixels.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        icon_commands.len(),
        1,
        "dropdown should render its chevron through the shared shell SVG asset"
    );
    let icon = icon_commands[0];
    assert_eq!(icon.frame.width, 16.0);
    assert_eq!(icon.frame.height, 16.0);
    assert!(icon
        .image_pixels
        .as_ref()
        .map(|image| !image.resource_key.starts_with("missing-icon:"))
        .unwrap_or(false));
}

#[test]
fn open_workbench_dropdown_uses_focus_border_and_keeps_popup_rows() {
    let mut node = dropdown_node(true);
    node.popup_open = true;
    node.structured_options = model_rc(vec![
        option("dropdown", true, false, true, false),
        option("option_a", false, true, false, false),
        option("option_b", false, false, false, true),
    ]);
    let bytes = paint_template_nodes_for_test(160, 140, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 160, 54, 8), DROPDOWN_FOCUS_BORDER);
    assert!(changed_pixel_count(&bytes, 160, 18, 44, 110, 78) > 0);
}

#[test]
fn workbench_dropdown_honors_declared_layout_offset() {
    let mut node = dropdown_node(false);
    node.layout_offset_x = 20.0;
    node.layout_offset_y = 12.0;
    let bytes = paint_template_nodes_for_test(160, 80, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 160, 88, 24), DROPDOWN_SURFACE);
    assert_eq!(pixel_at(&bytes, 160, 54, 8), [0, 0, 0, 255]);
}

#[test]
fn workbench_dropdown_applies_declared_visual_brightness() {
    let mut node = dropdown_node(false);
    node.label_brightness = 1.2;
    let expected_surface = scaled_test_color(DROPDOWN_SURFACE, 1.2);
    let expected_border = scaled_test_color(DROPDOWN_BORDER, 1.2);
    let bytes = paint_template_nodes_for_test(140, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 140, 88, 24), expected_surface);
    assert_eq!(pixel_at(&bytes, 140, 54, 8), expected_border);
}
