use super::*;

#[test]
fn code_component_variant_selects_runtime_code_text_style() {
    let code = TemplatePaneNodeData {
        component_variant: "outlined CODE compact".into(),
        ..TemplatePaneNodeData::default()
    };

    assert!(node_text_paint_style(&code).code);
    assert!(!node_text_paint_style(&TemplatePaneNodeData::default()).code);
}

#[test]
fn welcome_project_form_labels_emit_their_authored_nonempty_text() {
    for (control_id, expected_text) in [
        ("WelcomeProjectNameLabel", "Project name"),
        ("WelcomeLocationLabel", "Location"),
    ] {
        let node = TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Label".into(),
            text: expected_text.into(),
            text_tone: "muted".into(),
            font_size: 12.0,
            ..TemplatePaneNodeData::default()
        };
        let node_rect = FrameRect {
            x: 12.0,
            y: 16.0,
            width: 240.0,
            height: 24.0,
        };
        let clip = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 320.0,
            height: 200.0,
        };
        let mut commands = Vec::new();

        push_template_text_fallback_command(
            &mut commands,
            &node,
            &node_rect,
            &clip,
            3,
            None,
            false,
            false,
            1.0,
        );

        assert_eq!(commands.len(), 1, "{control_id} must emit one text command");
        assert_eq!(commands[0].text.as_deref(), Some(expected_text));
        assert_eq!(commands[0].clip_frame.as_ref(), Some(&clip));
        assert!(commands[0].frame.width > 0.0 && commands[0].frame.height > 0.0);
    }
}
#[test]
fn focused_text_input_paints_clipped_selection_and_caret_feedback() {
    use crate::ui::retained_host::host_contract::data::HostTextInputFocusData;

    let node = TemplatePaneNodeData {
        control_id: "NameField".into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        text: "placeholder".into(),
        ..TemplatePaneNodeData::default()
    };
    let focus = HostTextInputFocusData {
        control_id: "NameField".into(),
        value_text: "A🙂B".into(),
        caret_scalar_offset: Some(2),
        selection_anchor_scalar_offset: Some(0),
        ..HostTextInputFocusData::default()
    };
    let node_rect = FrameRect {
        x: 12.0,
        y: 16.0,
        width: 120.0,
        height: 24.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    let text_rect = text_rect_for_node(&node, &node_rect);
    let mut commands = Vec::new();
    push_template_text_fallback_command(
        &mut commands,
        &node,
        &node_rect,
        &clip,
        4,
        Some(&focus),
        false,
        false,
        1.0,
    );

    assert_eq!(
        commands.len(),
        3,
        "selection, text, and caret should all be painted"
    );
    assert_eq!(commands[0].background_color, Some([77, 137, 255, 102]));
    assert_eq!(commands[0].clip_frame.as_ref(), Some(&text_rect));
    assert!(commands[0].frame.x >= text_rect.x);
    assert!(commands[0].frame.x + commands[0].frame.width <= text_rect.x + text_rect.width);
    assert_eq!(commands[1].text.as_deref(), Some("A🙂B"));
    assert_eq!(commands[2].background_color, Some([232, 238, 247, 255]));
    assert_eq!(commands[2].clip_frame.as_ref(), Some(&text_rect));
    assert!(commands[2].frame.x >= text_rect.x);
    assert!(commands[2].frame.x <= text_rect.x + text_rect.width);
}

#[test]
fn unfocused_template_text_does_not_paint_edit_feedback() {
    let node = TemplatePaneNodeData {
        control_id: "NameField".into(),
        role: "InputField".into(),
        component_role: "input-field".into(),
        text: "A🙂B".into(),
        ..TemplatePaneNodeData::default()
    };
    let node_rect = FrameRect {
        x: 12.0,
        y: 16.0,
        width: 120.0,
        height: 24.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 200.0,
        height: 100.0,
    };
    let mut commands = Vec::new();
    push_template_text_fallback_command(
        &mut commands,
        &node,
        &node_rect,
        &clip,
        4,
        None,
        false,
        false,
        1.0,
    );
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].text.as_deref(), Some("A🙂B"));
}
