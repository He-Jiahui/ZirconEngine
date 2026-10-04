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
