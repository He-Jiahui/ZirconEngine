use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_alert_commands;
use super::support::positioned_alert_node;

#[test]
fn degenerate_alert_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node("WorkbenchInfoAlert", "Info", "info", 8.0, 6.0, 0.0, 32.0),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_inline_alert_omits_mark_and_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node("WorkbenchInfoAlert", "Info", "info", 8.0, 6.0, 1.0, 32.0),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert_eq!(commands.len(), 1, "only the inline surface should remain");
    assert!(commands.iter().all(|command| command.text.is_none()));
    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn narrow_toast_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node(
            "WorkbenchToastRoot",
            "Operation completed",
            "success",
            8.0,
            6.0,
            24.0,
            28.0,
        ),
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
fn short_toast_omits_text_action_and_marks() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 120.0,
        height: 4.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node(
            "WorkbenchToastRoot",
            "Operation completed",
            "success",
            8.0,
            6.0,
            120.0,
            4.0,
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert_eq!(commands.len(), 1, "only the toast surface should remain");
    assert!(commands.iter().all(|command| command.text.is_none()));
    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn alert_outside_its_clip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 9.0,
        y: 6.0,
        width: 120.0,
        height: 32.0,
    };
    let clip = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 120.0,
        height: 32.0,
    };
    let mut commands = Vec::new();

    assert!(push_alert_commands(
        &mut commands,
        &positioned_alert_node("WorkbenchInfoAlert", "Info", "info", 9.0, 6.0, 120.0, 32.0),
        &rect,
        &clip,
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
