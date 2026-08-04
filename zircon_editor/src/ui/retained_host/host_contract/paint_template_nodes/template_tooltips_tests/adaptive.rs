use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::{layout::tooltip_bubble_rect, push_tooltip_commands};
use super::support::tooltip_node;

#[test]
fn degenerate_tooltip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 78.0,
    };
    let mut commands = Vec::new();

    assert!(push_tooltip_commands(
        &mut commands,
        &tooltip_node(),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_tooltip_content_area_does_not_emit_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 1.0,
        height: 78.0,
    };
    let mut commands = Vec::new();

    assert!(push_tooltip_commands(
        &mut commands,
        &tooltip_node(),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
}

#[test]
fn narrow_tooltip_keeps_every_command_inside_its_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 24.0,
        height: 78.0,
    };
    let mut commands = Vec::new();

    assert!(push_tooltip_commands(
        &mut commands,
        &tooltip_node(),
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
fn tooltip_bubble_expands_for_content_and_stays_within_the_available_frame() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 260.0,
        height: 78.0,
    };
    let mut short = tooltip_node();
    short.text = "Open".into();
    short.label_text = "Open the selected document".into();
    let mut long = short.clone();
    long.text = "Resave selected editor assets".into();
    long.label_text = "Rebuild thumbnails and metadata for every selected editor asset".into();

    let short_bubble = tooltip_bubble_rect(&short, &rect);
    let long_bubble = tooltip_bubble_rect(&long, &rect);

    assert!(long_bubble.width > short_bubble.width);
    assert!(frame_is_within(&rect, &short_bubble));
    assert!(frame_is_within(&rect, &long_bubble));
    assert!(short_bubble.x > rect.x);
    assert!(long_bubble.width <= rect.width);
}

#[test]
fn tooltip_with_no_description_does_not_paint_placeholder_body_text() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 160.0,
        height: 48.0,
    };
    let mut node = tooltip_node();
    node.text = "Move".into();
    node.label_text.clear();
    let mut commands = Vec::new();

    assert!(push_tooltip_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Move")));
    assert!(
        commands
            .iter()
            .all(|command| command.text.as_deref() != Some("This is a tooltip")),
        "an icon tooltip without an authored description must not display sample text"
    );
}

#[test]
fn tooltip_without_description_uses_a_compact_bubble_height() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 160.0,
        height: 56.0,
    };
    let mut compact = tooltip_node();
    compact.text = "Move".into();
    compact.label_text.clear();
    let detailed = tooltip_node();

    let compact_bubble = tooltip_bubble_rect(&compact, &rect);
    let detailed_bubble = tooltip_bubble_rect(&detailed, &rect);

    assert!(compact_bubble.height < detailed_bubble.height);
    assert!(frame_is_within(&rect, &compact_bubble));
    assert!(frame_is_within(&rect, &detailed_bubble));
}

#[test]
fn tooltip_info_glyph_requires_a_declared_icon() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 160.0,
        height: 78.0,
    };
    let mut without_icon = Vec::new();

    assert!(push_tooltip_commands(
        &mut without_icon,
        &tooltip_node(),
        &rect,
        &rect,
        4,
        1.0,
    ));
    assert!(
        without_icon.iter().all(|command| command.z_index < 9),
        "a tooltip without an icon declaration must not emit the info glyph stem or dot"
    );

    let mut declared_icon = tooltip_node();
    declared_icon.icon_name = "info".into();
    let mut with_icon = Vec::new();

    assert!(push_tooltip_commands(
        &mut with_icon,
        &declared_icon,
        &rect,
        &rect,
        4,
        1.0,
    ));
    assert!(
        with_icon.iter().any(|command| command.z_index == 9),
        "an explicit icon declaration must retain the info glyph stem and dot"
    );
}

#[test]
fn fractional_tooltip_alignment_does_not_expand_its_logical_frame() {
    let rect = FrameRect {
        x: 8.2,
        y: 6.2,
        width: 24.6,
        height: 78.8,
    };
    let mut commands = Vec::new();

    assert!(push_tooltip_commands(
        &mut commands,
        &tooltip_node(),
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
fn short_tooltip_omits_shadow_text_arrow_and_icon() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 104.0,
        height: 4.0,
    };
    let mut commands = Vec::new();

    assert!(push_tooltip_commands(
        &mut commands,
        &tooltip_node(),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
    assert_eq!(commands.len(), 1, "only the bubble surface should remain");
    assert!(frame_is_within(&rect, &commands[0].frame));
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}
