use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_chip_commands;
use super::support::chip_node;

#[test]
fn degenerate_chip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_chip_commands(
        &mut commands,
        &chip_node("WorkbenchViewportMode", "Perspective"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_chip_content_area_does_not_emit_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_chip_commands(
        &mut commands,
        &chip_node("WorkbenchViewportMode", "Perspective"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
}

#[test]
fn narrow_chip_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_chip_commands(
        &mut commands,
        &chip_node("WorkbenchViewportMode", "Perspective"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(
        commands
            .iter()
            .all(|command| frame_is_within(&rect, &command.frame))
    );
}

#[test]
fn fractional_chip_alignment_does_not_expand_its_logical_frame() {
    let rect = FrameRect {
        x: 8.2,
        y: 6.2,
        width: 24.6,
        height: 28.8,
    };
    let mut commands = Vec::new();

    assert!(push_chip_commands(
        &mut commands,
        &chip_node("WorkbenchViewportMode", "Perspective"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(!commands.is_empty());
    assert!(
        commands
            .iter()
            .all(|command| frame_is_within(&rect, &command.frame))
    );
}

#[test]
fn short_chip_omits_text_and_chevron() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 104.0,
        height: 4.0,
    };
    let mut commands = Vec::new();

    assert!(push_chip_commands(
        &mut commands,
        &chip_node("WorkbenchViewportMode", "Perspective"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
    assert_eq!(commands.len(), 1, "only the chip surface should remain");
    assert!(frame_is_within(&rect, &commands[0].frame));
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
