use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::push_list_row_commands;
use super::support::{
    changed_pixel_count, list_node, list_node_with_flags, matching_pixel_count, pixel_at,
};

#[test]
fn selected_list_row_paints_muted_selected_fill_neutral_outline_and_navigation_adornment() {
    let bytes = paint_template_nodes_for_test(
        160,
        40,
        model_rc(vec![list_node_with_flags(true, false, false)]),
    );

    assert!(matching_pixel_count(&bytes, 160, 4, 4, 148, 32, PALETTE.border) > 300);
    assert!(matching_pixel_count(&bytes, 160, 4, 4, 148, 32, PALETTE.accent) > 0);
    assert!(matching_pixel_count(&bytes, 160, 36, 8, 80, 22, PALETTE.surface_pressed) > 1200);
    assert_eq!(
        matching_pixel_count(&bytes, 160, 36, 8, 80, 22, PALETTE.surface_selected),
        0
    );
    assert_eq!(pixel_at(&bytes, 160, 12, 30), PALETTE.surface_pressed);
    assert!(changed_pixel_count(&bytes, 160, 135, 12, 16, 16) > 0);
}

#[test]
fn checked_list_row_paints_right_check_with_muted_selected_fill() {
    let bytes = paint_template_nodes_for_test(160, 40, model_rc(vec![list_node(true, false)]));

    assert!(matching_pixel_count(&bytes, 160, 4, 4, 148, 32, PALETTE.border) > 300);
    assert!(matching_pixel_count(&bytes, 160, 4, 4, 148, 32, PALETTE.accent) > 0);
    assert!(matching_pixel_count(&bytes, 160, 36, 8, 80, 22, PALETTE.surface_pressed) > 1200);
    assert_eq!(
        matching_pixel_count(&bytes, 160, 36, 8, 80, 22, PALETTE.surface_selected),
        0
    );
    assert_eq!(pixel_at(&bytes, 160, 12, 30), PALETTE.surface_pressed);
    assert!(changed_pixel_count(&bytes, 160, 135, 12, 16, 16) > 0);
}

#[test]
fn focused_list_row_paints_border_without_hover_or_selected_fill() {
    let mut node = list_node(false, false);
    node.focused = true;

    let bytes = paint_template_nodes_for_test(160, 40, model_rc(vec![node]));

    assert!(matching_pixel_count(&bytes, 160, 4, 4, 148, 32, PALETTE.focus_ring) > 250);
    assert_eq!(
        matching_pixel_count(&bytes, 160, 36, 8, 80, 22, PALETTE.surface_hover),
        0
    );
    assert_eq!(
        matching_pixel_count(&bytes, 160, 36, 8, 80, 22, PALETTE.surface_pressed),
        0
    );
    assert_eq!(
        matching_pixel_count(&bytes, 160, 36, 8, 80, 22, PALETTE.surface_selected),
        0
    );
}

#[test]
fn disabled_list_row_keeps_background_empty_and_draws_disabled_adornment() {
    let node = list_node(false, true);
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 148.0,
        height: 32.0,
    };
    let mut commands = Vec::new();
    assert!(push_list_row_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0,
    ));
    assert!(commands
        .iter()
        .all(|command| command.background_color.is_none()));

    let bytes = paint_template_nodes_for_test(160, 40, model_rc(vec![node]));

    assert!(changed_pixel_count(&bytes, 160, 135, 12, 16, 16) > 0);
}

#[test]
fn workbench_list_row_adornments_paint_shell_asset_pixels() {
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 148.0,
        height: 32.0,
    };

    for node in [
        list_node(true, false),
        list_node(false, false),
        list_node(false, true),
    ] {
        let mut commands = Vec::new();
        assert!(push_list_row_commands(
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
        assert!(
            !icon_commands.is_empty(),
            "{} should render its trailing adornment through shell icon pixels",
            node.control_id
        );
        assert!(
            icon_commands.iter().all(|command| command
                .image_pixels
                .as_ref()
                .map(|image| !image.resource_key.starts_with("missing-icon:"))
                .unwrap_or(false)),
            "{} should not use missing-icon pixels for its trailing adornment",
            node.control_id
        );
        assert!(
            icon_commands.iter().any(|command| {
                (command.frame.width - 16.0).abs() < f32::EPSILON
                    && (command.frame.height - 16.0).abs() < f32::EPSILON
            }),
            "{} should use the Slate Icon16x16 trailing adornment size",
            node.control_id
        );
    }
}
