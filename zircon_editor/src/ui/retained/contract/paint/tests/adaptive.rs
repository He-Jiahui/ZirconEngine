use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_icon_button_commands;
use super::support::positioned_icon_node;

#[test]
fn degenerate_icon_button_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_icon_button_commands(
        &mut commands,
        &positioned_icon_node(
            "WorkbenchToolbarMenu",
            "zircon_editor_shell/toolbar/menu.svg",
            false,
            8.0,
            6.0,
            0.0,
            28.0,
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_icon_button_omits_the_glyph() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 8.0,
        height: 8.0,
    };
    let mut commands = Vec::new();

    assert!(push_icon_button_commands(
        &mut commands,
        &positioned_icon_node(
            "WorkbenchToolbarMenu",
            "zircon_editor_shell/toolbar/menu.svg",
            false,
            8.0,
            6.0,
            8.0,
            8.0,
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands
        .iter()
        .all(|command| command.image_pixels.is_none()));
    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn narrow_icon_button_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_icon_button_commands(
        &mut commands,
        &positioned_icon_node(
            "WorkbenchMiniAdd",
            "zircon_editor_shell/controls/add.svg",
            false,
            8.0,
            6.0,
            24.0,
            24.0,
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
fn offset_icon_button_overlapping_its_clip_keeps_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 30.0,
        height: 30.0,
    };
    let mut node = positioned_icon_node(
        "WorkbenchToolbarMenu",
        "zircon_editor_shell/toolbar/menu.svg",
        false,
        8.0,
        6.0,
        30.0,
        30.0,
    );
    node.layout_offset_x = 1.0;
    let mut commands = Vec::new();

    assert!(push_icon_button_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(!commands.is_empty());
    assert!(commands
        .iter()
        .all(|command| command.clip_frame.as_ref() == Some(&rect)));
}

#[test]
fn icon_button_outside_its_clip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 30.0,
        height: 30.0,
    };
    let clip = FrameRect {
        x: 48.0,
        ..rect.clone()
    };
    let mut commands = Vec::new();

    assert!(push_icon_button_commands(
        &mut commands,
        &positioned_icon_node(
            "WorkbenchToolbarMenu",
            "zircon_editor_shell/toolbar/menu.svg",
            false,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        ),
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
