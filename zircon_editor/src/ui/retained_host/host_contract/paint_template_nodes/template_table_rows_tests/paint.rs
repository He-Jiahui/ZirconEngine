use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style_selector::{
    WORKBENCH_TABLE_HEADER_BG as TABLE_HEADER_BG, WORKBENCH_TABLE_HEADER_TEXT as TABLE_HEADER_TEXT,
    WORKBENCH_TABLE_SELECTED_BG as TABLE_SELECTED_BG, WORKBENCH_TABLE_SEPARATOR as TABLE_SEPARATOR,
    WORKBENCH_TABLE_TAIL_BG as TABLE_TAIL_BG,
};
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::render_commands::HostPaintCommand;
use super::super::commands::{push_table_row_commands, push_table_row_text_commands};
use super::super::style::table_cell_color;
use super::support::{different_pixel_count, pixel_at, table_node};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn workbench_table_row_paints_selected_surface_and_action_glyph() {
    let bytes = paint_template_nodes_for_test(
        240,
        44,
        model_rc(vec![table_node("WorkbenchTableSelected", true)]),
    );

    assert_eq!(pixel_at(&bytes, 240, 5, 10), PALETTE.accent);
    assert_eq!(pixel_at(&bytes, 240, 8, 10), PALETTE.surface_pressed);
    assert_ne!(pixel_at(&bytes, 240, 8, 10), PALETTE.surface_selected);
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

    let mut hovered_commands = Vec::new();
    let mut hovered = table_node("WorkbenchTableHovered", false);
    hovered.hovered = true;

    let hovered_handled =
        push_table_row_commands(&mut hovered_commands, &hovered, &rect, &rect, 0, 1.0);

    assert!(hovered_handled);
    assert!(table_row_action_asset_count(&hovered_commands) > 0);

    let mut selected_commands = Vec::new();
    let selected = table_node("WorkbenchTableSelected", true);

    let selected_handled =
        push_table_row_commands(&mut selected_commands, &selected, &rect, &rect, 0, 1.0);

    assert!(selected_handled);
    assert!(table_row_action_asset_count(&selected_commands) > 0);
}

fn table_row_action_asset_count(commands: &[HostPaintCommand]) -> usize {
    commands
        .iter()
        .filter_map(|command| command.image_pixels.as_ref())
        .filter(|image| image.resource_key.contains("more-horizontal.svg"))
        .count()
}
