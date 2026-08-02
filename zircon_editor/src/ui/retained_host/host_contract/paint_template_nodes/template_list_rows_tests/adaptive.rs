use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::super::template_row_metrics::workbench_row_metrics;
use super::super::push_list_row_commands;
use super::support::list_node;

#[test]
fn degenerate_list_row_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_list_row_commands(
        &mut commands,
        &list_node(true, false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn selection_indicator_stays_within_a_narrow_list_row() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.5,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_list_row_commands(
        &mut commands,
        &list_node(true, false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    let indicator = commands
        .iter()
        .find(|command| command.background_color == Some(PALETTE.accent))
        .expect("selected list rows should retain their selection indicator");
    assert_eq!(indicator.frame.width, rect.width);
}

#[test]
fn narrow_list_row_surface_radius_stays_within_the_row_extent() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.5,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_list_row_commands(
        &mut commands,
        &list_node(true, false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    let surface = commands
        .iter()
        .find(|command| command.background_color == Some(PALETTE.surface_pressed))
        .expect("selected list rows should retain their surface");
    assert!(surface.corner_radius <= rect.width * 0.5);
}

#[test]
fn collapsed_list_row_text_slot_does_not_emit_a_one_pixel_text_command() {
    let metrics = workbench_row_metrics();
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: metrics.text_inset_x + metrics.right_reserve,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_list_row_commands(
        &mut commands,
        &list_node(false, false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
}

#[test]
fn narrow_list_row_elides_an_adornment_that_would_escape_its_frame() {
    let metrics = workbench_row_metrics();
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: metrics.list_adornment_right_inset + metrics.list_adornment_size - 0.25,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_list_row_commands(
        &mut commands,
        &list_node(true, false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(
        commands
            .iter()
            .all(|command| command.image_pixels.is_none())
    );
}

#[test]
fn short_list_row_elides_an_adornment_that_would_escape_its_frame() {
    let metrics = workbench_row_metrics();
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: metrics.list_adornment_right_inset + metrics.list_adornment_size,
        height: metrics.list_adornment_size - 0.25,
    };
    let mut commands = Vec::new();

    assert!(push_list_row_commands(
        &mut commands,
        &list_node(true, false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(
        commands
            .iter()
            .all(|command| command.image_pixels.is_none())
    );
}
