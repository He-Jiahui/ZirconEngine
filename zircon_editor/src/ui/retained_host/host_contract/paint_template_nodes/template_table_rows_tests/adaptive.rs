use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_table_row_commands;
use super::support::table_node;

#[test]
fn degenerate_table_row_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_table_row_commands(
        &mut commands,
        &table_node("WorkbenchTableHeader", false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_table_content_area_does_not_emit_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_table_row_commands(
        &mut commands,
        &table_node("WorkbenchTableHeader", false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
}

#[test]
fn fully_clipped_table_row_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 200.0,
        height: 28.0,
    };
    let clip = FrameRect {
        x: 240.0,
        y: 0.0,
        width: 80.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    assert!(push_table_row_commands(
        &mut commands,
        &table_node("WorkbenchTableHeader", false),
        &rect,
        &clip,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn partially_clipped_table_row_keeps_only_clipped_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 200.0,
        height: 28.0,
    };
    let clip = FrameRect {
        x: 16.0,
        y: 8.0,
        width: 60.0,
        height: 22.0,
    };
    let mut commands = Vec::new();

    assert!(push_table_row_commands(
        &mut commands,
        &table_node("WorkbenchTableHeader", false),
        &rect,
        &clip,
        4,
        1.0,
    ));

    assert!(!commands.is_empty());
    assert!(commands.iter().all(|command| command
        .clip_frame
        .as_ref()
        .is_some_and(|clip_frame| frame_is_within(&clip, clip_frame))));
}

#[test]
fn narrow_table_row_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_table_row_commands(
        &mut commands,
        &table_node("WorkbenchTableHeader", false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn short_table_row_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 200.0,
        height: 0.5,
    };
    let mut commands = Vec::new();

    assert!(push_table_row_commands(
        &mut commands,
        &table_node("WorkbenchTableHeader", false),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
