use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::push_field_commands;
use super::support::positioned_field_node;

#[test]
fn degenerate_field_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 28.0,
    };
    let mut commands = Vec::new();

    assert!(push_field_commands(
        &mut commands,
        &positioned_field_node("WorkbenchInputText", "Name", 8.0, 6.0, 0.0, 28.0),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_field_content_omits_text_and_search_glyph() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 28.0,
    };
    let mut search = positioned_field_node("SearchEdited", "", 8.0, 6.0, 1.0, 28.0);
    search.text = "Search".into();
    let mut commands = Vec::new();

    assert!(push_field_commands(
        &mut commands,
        &search,
        &rect,
        &rect,
        4,
        1.0
    ));

    assert_eq!(commands.len(), 1, "only the field surface should remain");
    assert!(commands.iter().all(|command| command.text.is_none()));
    assert!(commands
        .iter()
        .all(|command| command.image_pixels.is_none()));
    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn narrow_fields_keep_every_command_inside_the_field_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 28.0,
    };
    let mut commands = Vec::new();
    let mut search = positioned_field_node("SearchEdited", "", 8.0, 6.0, 24.0, 28.0);
    search.text = "Search".into();

    assert!(push_field_commands(
        &mut commands,
        &search,
        &rect,
        &rect,
        4,
        1.0
    ));

    assert!(commands
        .iter()
        .all(|command| frame_is_within(&rect, &command.frame)));
}

#[test]
fn fractional_field_alignment_does_not_expand_its_logical_frame() {
    let rect = FrameRect {
        x: 8.2,
        y: 6.2,
        width: 84.6,
        height: 28.8,
    };
    let mut commands = Vec::new();

    assert!(push_field_commands(
        &mut commands,
        &positioned_field_node("WorkbenchInputText", "Name", 8.2, 6.2, 84.6, 28.8),
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
fn offset_field_outside_its_clip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 80.0,
        height: 28.0,
    };
    let mut field = positioned_field_node("WorkbenchInputText", "Name", 8.0, 6.0, 80.0, 28.0);
    field.layout_offset_x = 1.0;
    let mut commands = Vec::new();

    assert!(push_field_commands(
        &mut commands,
        &field,
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn short_stepper_field_omits_text_and_stepper_glyphs() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 80.0,
        height: 4.0,
    };
    let mut commands = Vec::new();

    assert!(push_field_commands(
        &mut commands,
        &positioned_field_node("WorkbenchInputStepper", "42", 8.0, 6.0, 80.0, 4.0),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert_eq!(commands.len(), 1, "only the field surface should remain");
    assert!(commands.iter().all(|command| command.text.is_none()));
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
