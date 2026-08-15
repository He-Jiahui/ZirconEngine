use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_selection_control_commands;
use super::support::node_with_role;

#[test]
fn degenerate_selection_control_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_selection_control_commands(
        &mut commands,
        &node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOff"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_selection_control_does_not_expand_mark_or_label() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 28.0,
    };
    let mut node = node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOn");
    node.checked = true;
    node.text = "Enabled".into();
    let mut commands = Vec::new();

    assert!(push_selection_control_commands(
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
fn narrow_toggle_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 28.0,
    };
    let mut node = node_with_role("Toggle", "toggle", "WorkbenchToggleOn");
    node.checked = true;
    node.text = "Enabled".into();
    let mut commands = Vec::new();

    assert!(push_selection_control_commands(
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

#[test]
fn short_radio_omits_mark_dot_and_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 80.0,
        height: 4.0,
    };
    let mut node = node_with_role("Radio", "radio", "WorkbenchRadioOn");
    node.checked = true;
    node.text = "Selected".into();
    let mut commands = Vec::new();

    assert!(push_selection_control_commands(
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
fn selection_control_outside_its_clip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 9.0,
        y: 6.0,
        width: 80.0,
        height: 28.0,
    };
    let clip = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 80.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_selection_control_commands(
        &mut commands,
        &node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOff"),
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
