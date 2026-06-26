use super::super::super::super::data::FrameRect;
use super::super::super::template_inspector_row_geometry::{
    shadow_check_content_offset_x, shadow_check_rect,
};
use super::super::super::template_inspector_row_kind::{
    COMPONENT_PROPERTY_SLOT_03, MATERIAL_PROPERTY_ROW,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::push_inspector_row_commands;
use super::super::style::{resource_field_background, resource_field_border, resource_value_color};
use super::support::{
    changed_pixel_count, inspector_node, pixel_at, resolved_background_and_border,
};
use crate::ui::layouts::common::model_rc;

#[test]
fn receive_shadows_row_paints_checked_box_without_full_field() {
    let mut row = inspector_node(COMPONENT_PROPERTY_SLOT_03, "Receive Shadows", "true");
    row.layout_content_offset_x = 34.0;
    let bytes = paint_template_nodes_for_test(320, 48, model_rc(vec![row.clone()]));
    let rect = FrameRect {
        x: row.frame.x,
        y: row.frame.y,
        width: row.frame.width,
        height: row.frame.height,
    };

    assert_eq!(shadow_check_content_offset_x(&row), 34.0);
    assert_eq!(shadow_check_rect(&row, &rect).x, 158.0);
    assert!(changed_pixel_count(&bytes, 320, 156, 14, 20, 18) > 0);
    assert_eq!(pixel_at(&bytes, 320, 250, 16), [0, 0, 0, 255]);
}

#[test]
fn cast_shadows_select_uses_declared_field_and_value_tones() {
    let mut row = inspector_node(MATERIAL_PROPERTY_ROW, "Cast Shadows", "false");
    row.button_style = resolved_background_and_border([40, 46, 50, 255], [52, 61, 67, 255]);
    row.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(181, 192, 197);

    assert_eq!(resource_field_background(&row), [40, 46, 50, 255]);
    assert_eq!(resource_field_border(&row), [52, 61, 67, 255]);
    assert_eq!(resource_value_color(&row), [181, 192, 197, 255]);
}

#[test]
fn receive_shadows_row_uses_legacy_checkbox_offset_without_declaration() {
    let row = inspector_node(COMPONENT_PROPERTY_SLOT_03, "Receive Shadows", "true");
    let rect = FrameRect {
        x: row.frame.x,
        y: row.frame.y,
        width: row.frame.width,
        height: row.frame.height,
    };

    assert_eq!(shadow_check_content_offset_x(&row), 28.0);
    assert_eq!(shadow_check_rect(&row, &rect).x, 152.0);
}

#[test]
fn checked_shadow_row_paints_shell_check_asset_pixels() {
    let row = inspector_node(COMPONENT_PROPERTY_SLOT_03, "Receive Shadows", "true");
    let rect = FrameRect {
        x: row.frame.x,
        y: row.frame.y,
        width: row.frame.width,
        height: row.frame.height,
    };
    let mut commands = Vec::new();

    assert!(push_inspector_row_commands(
        &mut commands,
        &row,
        &rect,
        &rect,
        0,
        1.0,
    ));

    let assets = commands
        .iter()
        .filter_map(|command| command.image_pixels.as_ref())
        .collect::<Vec<_>>();
    assert!(
        !assets.is_empty(),
        "checked inspector checkbox should render its tick through shell icon pixels"
    );
    assert!(
        assets
            .iter()
            .all(|image| !image.resource_key.starts_with("missing-icon:")),
        "checked inspector checkbox should not use missing-icon pixels"
    );
}
