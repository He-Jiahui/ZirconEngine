use super::*;
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;

#[test]
fn template_node_text_command_uses_shared_runtime_line_height() {
    let mut commands = Vec::new();
    let frame = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 80.0,
        height: 20.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 96.0,
        height: 32.0,
    };

    push_text_command(
        &mut commands,
        &frame,
        &clip,
        7,
        "Caption".to_string(),
        [226, 230, 232, 255],
        10.666_667,
        UiTextRunPaintStyle {
            code: true,
            ..UiTextRunPaintStyle::default()
        },
        1.0,
    );

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].frame, frame);
    assert_eq!(commands[0].clip_frame.as_ref(), Some(&clip));
    assert_eq!(commands[0].z_index, 7);
    assert_eq!(commands[0].text.as_deref(), Some("Caption"));
    assert_eq!(
        commands[0].text_layout_policy,
        HostTextLayoutPolicy::WordWrap
    );
    assert!(commands[0].text_style.code);
    assert_eq!(
        commands[0].line_height,
        template_node_text_line_height(10.666_667)
    );
}

#[test]
fn template_node_text_command_skips_an_empty_or_unpaintable_slot() {
    let mut commands = Vec::new();
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 0.0,
        height: 20.0,
    };

    push_text_command(
        &mut commands,
        &rect,
        &rect,
        7,
        "Caption".to_string(),
        [226, 230, 232, 255],
        10.666_667,
        UiTextRunPaintStyle::default(),
        1.0,
    );
    let populated_rect = FrameRect {
        width: 80.0,
        ..rect.clone()
    };
    push_text_command(
        &mut commands,
        &populated_rect,
        &populated_rect,
        8,
        String::new(),
        [226, 230, 232, 255],
        10.666_667,
        UiTextRunPaintStyle::default(),
        1.0,
    );

    assert!(commands.is_empty());
}

#[test]
fn text_outside_clip_does_not_emit_a_command() {
    let mut commands = Vec::new();
    let rect = FrameRect {
        x: 20.0,
        y: 20.0,
        width: 10.0,
        height: 10.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    };

    push_text_command(
        &mut commands,
        &rect,
        &clip,
        7,
        "Outside".to_string(),
        [226, 230, 232, 255],
        10.666_667,
        UiTextRunPaintStyle::default(),
        1.0,
    );

    assert!(commands.is_empty());
}
