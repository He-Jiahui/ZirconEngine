use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::{
    WORKBENCH_TABLE_HEADER_BG as TABLE_HEADER_BG, WORKBENCH_TABLE_HEADER_TEXT as TABLE_HEADER_TEXT,
    WORKBENCH_TABLE_SELECTED_BG as TABLE_SELECTED_BG, WORKBENCH_TABLE_SEPARATOR as TABLE_SEPARATOR,
    WORKBENCH_TABLE_TAIL_BG as TABLE_TAIL_BG,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::commands::{push_table_row_commands, push_table_row_text_commands};
use super::super::style::table_cell_color;
use super::support::{different_pixel_count, matching_pixel_count, pixel_at, table_node};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn workbench_table_row_paints_muted_selected_fill_and_action_glyph() {
    let bytes = paint_template_nodes_for_test(
        240,
        44,
        model_rc(vec![table_node("WorkbenchTableSelected", true)]),
    );

    assert!(matching_pixel_count(&bytes, 240, 4, 4, 232, 28, PALETTE.border) > 400);
    assert_eq!(
        matching_pixel_count(&bytes, 240, 4, 4, 232, 28, PALETTE.accent),
        0
    );
    assert!(matching_pixel_count(&bytes, 240, 8, 8, 120, 18, PALETTE.surface_pressed) > 1500);
    assert_eq!(
        matching_pixel_count(&bytes, 240, 8, 8, 120, 18, PALETTE.surface_selected),
        0
    );
    assert_eq!(
        matching_pixel_count(&bytes, 240, 4, 30, 232, 2, TABLE_SEPARATOR),
        0
    );
    assert_eq!(pixel_at(&bytes, 240, 8, 10), PALETTE.surface_pressed);
    assert!(different_pixel_count(&bytes, 240, 220, 15, 14, 14, TABLE_SELECTED_BG) > 0);
    assert!(different_pixel_count(&bytes, 240, 14, 11, 90, 14, TABLE_SELECTED_BG) > 0);
}

#[test]
fn workbench_table_header_paints_recessed_surface_separator_and_gear() {
    let node = table_node("WorkbenchTableHeader", false);
    assert_eq!(table_cell_color(&node, 0), TABLE_HEADER_TEXT);
    let bytes = paint_template_nodes_for_test(240, 44, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 240, 8, 10), TABLE_HEADER_BG);
    assert_eq!(pixel_at(&bytes, 240, 8, 31), TABLE_SEPARATOR);
    assert!(different_pixel_count(&bytes, 240, 220, 15, 14, 14, TABLE_HEADER_BG) > 0);
}

#[test]
fn workbench_table_tail_uses_recessed_table_surface() {
    let bytes = paint_template_nodes_for_test(
        240,
        44,
        model_rc(vec![table_node("WorkbenchTableTail", false)]),
    );

    assert_eq!(pixel_at(&bytes, 240, 8, 10), TABLE_TAIL_BG);
    assert!(different_pixel_count(&bytes, 240, 14, 11, 90, 14, TABLE_TAIL_BG) > 0);
}

#[test]
fn workbench_table_row_text_path_skips_rows_with_full_table_paint() {
    let node = table_node("WorkbenchAssetBrowserAssetRow01", false);
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    let handled = push_table_row_text_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    assert!(!handled);
    assert!(commands.is_empty());
}

#[test]
fn workbench_table_actions_paint_shell_asset_pixels() {
    let node = table_node("WorkbenchTableSelected", true);
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    let handled = push_table_row_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    assert!(handled);
    let asset_pixels = commands
        .iter()
        .filter_map(|command| command.image_pixels.as_ref())
        .collect::<Vec<_>>();
    assert!(
        !asset_pixels.is_empty(),
        "table row action should render through the shared shell SVG asset path"
    );
    assert!(
        asset_pixels
            .iter()
            .all(|image| !image.resource_key.starts_with("missing-icon:")),
        "table row action should not fall back to missing-icon pixels"
    );

    let action_image =
        table_action_image_command(&commands, "more-horizontal.svg").expect("row action image");
    assert_frame_size(&action_image.frame, 16.0, 16.0);

    let action_slot =
        table_action_button_slot_command(&commands).expect("row action should paint button slot");
    assert_frame_size(&action_slot.frame, 20.0, 20.0);
    assert_eq!(action_slot.background_color, Some(PALETTE.surface_hover));
    assert_eq!(action_slot.border_color, Some(PALETTE.border));
    assert_eq!(action_slot.border_width, 1.0);
    assert_eq!(action_slot.corner_radius, 4.0);
    assert_eq!(action_slot.z_index + 1, action_image.z_index);
    assert_rect_contains(&action_slot.frame, &action_image.frame);
}

#[test]
fn workbench_table_row_action_stays_hidden_until_marked_or_hot() {
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };
    let mut neutral_commands = Vec::new();
    let neutral = table_node("WorkbenchTableNeutral", false);

    let neutral_handled =
        push_table_row_commands(&mut neutral_commands, &neutral, &rect, &rect, 0, 1.0);

    assert!(neutral_handled);
    assert_eq!(table_row_action_asset_count(&neutral_commands), 0);
    assert!(table_action_button_slot_command(&neutral_commands).is_none());

    let mut hovered_commands = Vec::new();
    let mut hovered = table_node("WorkbenchTableHovered", false);
    hovered.hovered = true;

    let hovered_handled =
        push_table_row_commands(&mut hovered_commands, &hovered, &rect, &rect, 0, 1.0);

    assert!(hovered_handled);
    assert!(table_row_action_asset_count(&hovered_commands) > 0);
    assert!(table_action_button_slot_command(&hovered_commands).is_some());

    let mut selected_commands = Vec::new();
    let selected = table_node("WorkbenchTableSelected", true);

    let selected_handled =
        push_table_row_commands(&mut selected_commands, &selected, &rect, &rect, 0, 1.0);

    assert!(selected_handled);
    assert!(table_row_action_asset_count(&selected_commands) > 0);
    assert!(table_action_button_slot_command(&selected_commands).is_some());
}

#[test]
fn workbench_table_header_action_uses_standard_icon_button_slot() {
    let node = table_node("WorkbenchTableHeader", false);
    let rect = FrameRect {
        x: 4.0,
        y: 4.0,
        width: 232.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    let handled = push_table_row_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    assert!(handled);
    let action_image =
        table_action_image_command(&commands, "settings.svg").expect("header action image");
    assert_frame_size(&action_image.frame, 16.0, 16.0);

    let action_slot = table_action_button_slot_command(&commands)
        .expect("header action should paint button slot");
    assert_frame_size(&action_slot.frame, 20.0, 20.0);
    assert_eq!(action_slot.background_color, Some(PALETTE.surface_pressed));
    assert_eq!(action_slot.border_color, Some(PALETTE.border));
    assert_eq!(action_slot.border_width, 1.0);
    assert_rect_contains(&action_slot.frame, &action_image.frame);
}

fn table_row_action_asset_count(commands: &[HostPaintCommand]) -> usize {
    commands
        .iter()
        .filter_map(|command| command.image_pixels.as_ref())
        .filter(|image| image.resource_key.contains("more-horizontal.svg"))
        .count()
}

fn table_action_image_command<'a>(
    commands: &'a [HostPaintCommand],
    resource_key: &str,
) -> Option<&'a HostPaintCommand> {
    commands.iter().find(|command| {
        command
            .image_pixels
            .as_ref()
            .is_some_and(|image| image.resource_key.contains(resource_key))
    })
}

fn table_action_button_slot_command(commands: &[HostPaintCommand]) -> Option<&HostPaintCommand> {
    commands.iter().find(|command| {
        command.frame.width == 20.0
            && command.frame.height == 20.0
            && matches!(
                command.background_color,
                Some(color) if color == PALETTE.surface_hover || color == PALETTE.surface_pressed
            )
            && command.border_color == Some(PALETTE.border)
    })
}

fn assert_frame_size(frame: &FrameRect, width: f32, height: f32) {
    assert_eq!(frame.width, width);
    assert_eq!(frame.height, height);
}

fn assert_rect_contains(outer: &FrameRect, inner: &FrameRect) {
    assert!(outer.x <= inner.x);
    assert!(outer.y <= inner.y);
    assert!(outer.x + outer.width >= inner.x + inner.width);
    assert!(outer.y + outer.height >= inner.y + inner.height);
}
