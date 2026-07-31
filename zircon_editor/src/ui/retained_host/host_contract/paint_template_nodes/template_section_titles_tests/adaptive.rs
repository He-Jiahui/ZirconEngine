use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_section_title_commands;
use super::support::title_node;

#[test]
fn degenerate_section_title_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 30.0,
    };
    let mut commands = Vec::new();

    assert!(push_section_title_commands(
        &mut commands,
        &title_node("WorkbenchInspectorTitle", "Props"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_section_title_content_area_does_not_emit_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 30.0,
    };
    let mut commands = Vec::new();

    assert!(push_section_title_commands(
        &mut commands,
        &title_node("WorkbenchInspectorTitle", "Props"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
}

#[test]
fn narrow_section_title_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 30.0,
    };
    let mut commands = Vec::new();

    assert!(push_section_title_commands(
        &mut commands,
        &title_node("WorkbenchInspectorTitle", "Props"),
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
fn fractional_section_title_alignment_does_not_expand_its_logical_frame() {
    let rect = FrameRect {
        x: 8.2,
        y: 6.2,
        width: 24.6,
        height: 30.8,
    };
    let mut commands = Vec::new();

    assert!(push_section_title_commands(
        &mut commands,
        &title_node("WorkbenchInspectorTitle", "Props"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(!commands.is_empty());
    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn short_section_title_omits_icon_and_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 104.0,
        height: 4.0,
    };
    let mut commands = Vec::new();

    assert!(push_section_title_commands(
        &mut commands,
        &title_node("WorkbenchInspectorTitle", "Props"),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
    assert_eq!(commands.len(), 1, "only the title surface should remain");
    assert!(frame_is_within(&rect, &commands[0].frame));
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
