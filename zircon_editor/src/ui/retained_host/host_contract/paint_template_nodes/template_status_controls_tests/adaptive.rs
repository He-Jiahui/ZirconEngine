use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_status_control_commands;
use super::support::{status_chip_node, status_icon_node, status_node};

#[test]
fn degenerate_status_chip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 30.0,
    };
    let mut commands = Vec::new();

    assert!(push_status_control_commands(
        &mut commands,
        &status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_status_chip_content_area_does_not_emit_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 30.0,
    };
    let mut commands = Vec::new();

    assert!(push_status_control_commands(
        &mut commands,
        &status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
}

#[test]
fn narrow_status_controls_keep_every_command_inside_their_frame() {
    for node in [
        status_chip_node("WorkbenchStatusGrid", "Grid: 10 cm"),
        status_icon_node("WorkbenchStatusTarget"),
        status_node("WorkbenchStatusReady", "Ready", 24.0, 30.0),
    ] {
        let rect = FrameRect {
            x: 8.0,
            y: 6.0,
            width: 24.0,
            height: 30.0,
        };
        let mut commands = Vec::new();

        assert!(push_status_control_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            4,
            1.0,
        ));

        assert!(commands
            .iter()
            .all(|command| frame_is_within(&rect, &command.frame)));
    }
}

#[test]
fn offset_status_signal_does_not_escape_its_parent_frame() {
    let mut node = status_node("WorkbenchStatusReady", "Ready", 72.0, 30.0);
    node.layout_offset_x = 100.0;
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 72.0,
        height: 30.0,
    };
    let mut commands = Vec::new();

    assert!(push_status_control_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn short_status_signal_does_not_emit_partial_marker_or_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 72.0,
        height: 4.0,
    };
    let mut commands = Vec::new();

    assert!(push_status_control_commands(
        &mut commands,
        &status_node("WorkbenchStatusReady", "Ready", 72.0, 4.0),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
