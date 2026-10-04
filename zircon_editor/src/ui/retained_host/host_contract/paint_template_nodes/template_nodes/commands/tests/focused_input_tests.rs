use super::*;
use crate::ui::retained_host::host_contract::data::TemplateNodeFrameData;

fn commands_for(
    control_id: &str,
    role: &str,
    focus_id: &str,
    draft: &str,
) -> Vec<HostPaintCommand> {
    let node = TemplatePaneNodeData {
        control_id: control_id.into(),
        role: role.into(),
        component_role: "input-field".into(),
        value_text: "0.00".into(),
        font_size: 18.0,
        frame: TemplateNodeFrameData {
            x: 1596.0,
            y: 834.0,
            width: 84.0,
            height: 41.0,
        },
        has_clip_frame: true,
        clip_frame: TemplateNodeFrameData {
            x: 1596.0,
            y: 834.0,
            width: 84.0,
            height: 41.0,
        },
        ..TemplatePaneNodeData::default()
    };
    let focus = HostTextInputFocusData {
        control_id: focus_id.into(),
        value_text: draft.into(),
        caret_scalar_offset: Some(draft.chars().count()),
        selection_anchor_scalar_offset: Some(0),
        ..HostTextInputFocusData::default()
    };
    let clip = FrameRect {
        x: 1500.0,
        y: 800.0,
        width: 220.0,
        height: 100.0,
    };
    let mut commands = Vec::new();
    push_template_node_commands(
        &mut commands,
        &node,
        &FrameRect::default(),
        &clip,
        Some(&focus),
        5,
    );
    assert_eq!(
        node.value_text.as_str(),
        "0.00",
        "draft painting must not mutate model values"
    );
    assert!(!node.focused, "focus styling is local paint state");
    commands
}

#[test]
fn specialized_axis_and_regular_input_use_live_draft_with_clipped_edit_feedback() {
    for id in ["WorkbenchTransformPositionX", "WorkbenchInputDefault"] {
        let commands = commands_for(id, "InputField", id, "1.25");
        let texts: Vec<_> = commands
            .iter()
            .filter(|command| command.text.is_some())
            .collect();
        assert_eq!(
            texts.len(),
            1,
            "no committed value beneath or above the draft: {id}"
        );
        assert_eq!(texts[0].text.as_deref(), Some("1.25"));
        let baseline = commands_for(id, "InputField", "OtherField", "1.25");
        let idle_text = baseline
            .iter()
            .find(|command| command.text.is_some())
            .unwrap();
        assert_eq!(
            texts[0].font_size, idle_text.font_size,
            "specialized font metrics are retained"
        );
        assert_eq!(
            texts[0].frame, idle_text.frame,
            "draft must not move the specialized text slot"
        );
        for color in [[77, 137, 255, 102], [232, 238, 247, 255]] {
            let feedback: Vec<_> = commands
                .iter()
                .filter(|command| command.background_color == Some(color))
                .collect();
            assert_eq!(feedback.len(), 1, "exactly one selection/caret for {id}");
            let clip = feedback[0].clip_frame.as_ref().unwrap();
            assert!(clip.x >= 1596.0 && clip.right() <= 1680.0);
            assert!(clip.y >= 834.0 && clip.bottom() <= 875.0);
        }
    }
}

#[test]
fn empty_focused_draft_does_not_restore_committed_numeric_text() {
    let commands = commands_for(
        "WorkbenchTransformPositionX",
        "InputField",
        "WorkbenchTransformPositionX",
        "",
    );
    assert!(commands
        .iter()
        .all(|command| command.text.as_deref().is_none_or(str::is_empty)));
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.background_color == Some([232, 238, 247, 255]))
            .count(),
        1
    );
}

#[test]
fn another_control_focus_preserves_committed_value_and_has_no_edit_feedback() {
    let commands = commands_for(
        "WorkbenchTransformPositionX",
        "InputField",
        "OtherField",
        "1.25",
    );
    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("0.00")));
    assert!(commands
        .iter()
        .all(|command| command.background_color != Some([232, 238, 247, 255])));
}
