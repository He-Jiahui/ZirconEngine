use super::*;
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;
use crate::ui::retained_host::primitives::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

fn message_model(values: &[&str]) -> ModelRc<SharedString> {
    ModelRc::from(Rc::new(VecModel::from(
        values
            .iter()
            .map(|value| SharedString::from(*value))
            .collect::<Vec<_>>(),
    )))
}

#[test]
fn message_values_are_role_routed_and_wrapped() {
    let node = TemplatePaneNodeData {
        collection_items: message_model(&["user|Question", "agent|Answer"]),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 240.0,
        height: 160.0,
    };
    let mut commands = Vec::new();

    push_agent_chat_content(&mut commands, &node, &rect, &rect, 4, 1.0);

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].text.as_deref(), Some("Answer"));
    assert_eq!(commands[1].text.as_deref(), Some("Question"));
    assert!(commands
        .iter()
        .all(|command| command.text_layout_policy == HostTextLayoutPolicy::WordWrap));
    assert!(commands[0].frame.x < commands[1].frame.x);
}

#[test]
fn plain_node_text_is_used_when_no_message_model_is_present() {
    let node = TemplatePaneNodeData {
        text: "Fallback response".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    push_agent_chat_content(&mut commands, &node, &rect, &rect, 0, 1.0);

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].text.as_deref(), Some("Fallback response"));
}

#[test]
fn authored_agent_chat_paint_scales_once_and_fallback_stays_physical() {
    use crate::ui::retained_host::host_contract::paint_theme::{
        current_host_metrics, enter_host_paint_theme_scope,
        host_paint_theme_snapshot_at_scale_for_test,
    };
    for scale in [1.0, 1.5] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));

        let rect = FrameRect {
            width: 360.0,
            height: 240.0,
            ..Default::default()
        };
        for (font_size, expected) in [
            (12.0, 12.0 * scale),
            (0.0, current_host_metrics().font_small),
        ] {
            let node = TemplatePaneNodeData {
                text: "DPI chat".into(),
                font_size,
                ..Default::default()
            };
            let mut commands = Vec::new();
            push_agent_chat_content(&mut commands, &node, &rect, &rect, 0, 1.0);
            let message = commands
                .iter()
                .find(|command| command.text.as_deref() == Some("DPI chat"))
                .expect("message paint command");
            assert_eq!(message.font_size, expected);
            assert_eq!(
                message.line_height,
                current_host_metrics().line_height(expected).max(expected)
            );
        }
    }
}
