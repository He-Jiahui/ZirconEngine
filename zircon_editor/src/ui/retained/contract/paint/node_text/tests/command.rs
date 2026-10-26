use super::*;
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;

#[test]
fn label_paint_command_keeps_full_layer_mask_lane_and_original_clip() {
    use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
    let node = TemplatePaneNodeData {
        role: "Label".into(),
        component_role: "label".into(),
        text: "Layer mask".into(),
        font_size: 13.333_333,
        ..TemplatePaneNodeData::default()
    };
    let frame = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 72.0,
        height: 24.0,
    };
    let clip = FrameRect {
        width: 48.0,
        ..frame.clone()
    };
    let mut commands = Vec::new();

    super::super::push_template_text_fallback_command(
        &mut commands,
        &node,
        &frame,
        &clip,
        7,
        None,
        false,
        false,
        1.0,
    );

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].text.as_deref(), Some("Layer mask"));
    assert_eq!(commands[0].frame.x, frame.x);
    assert_eq!(commands[0].frame.width, frame.width);
    assert_eq!(commands[0].clip_frame.as_ref(), Some(&clip));
}

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
