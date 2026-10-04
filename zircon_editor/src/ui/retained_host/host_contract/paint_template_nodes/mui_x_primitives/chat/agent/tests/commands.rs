use std::rc::Rc;

use super::*;
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;
use crate::ui::retained_host::primitives::{ModelRc, SharedString, VecModel};

fn messages(values: &[&str]) -> ModelRc<SharedString> {
    ModelRc::from(Rc::new(VecModel::from(
        values
            .iter()
            .map(|value| SharedString::from(*value))
            .collect::<Vec<_>>(),
    )))
}

#[test]
fn agent_chat_paints_projected_message_text_inside_both_bubbles() {
    let node = TemplatePaneNodeData {
        collection_items: messages(&[
            "user|Review the narrow shell",
            "agent|Checking responsive rules",
        ]),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 240.0,
        height: 160.0,
    };
    let mut commands = Vec::new();

    push_agent_chat(&mut commands, &node, &rect, &rect, 10, 1.0);

    let text_commands = commands
        .iter()
        .filter(|command| command.text.is_some())
        .collect::<Vec<_>>();
    assert_eq!(text_commands.len(), 2);
    assert!(text_commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Review the narrow shell")));
    assert!(text_commands
        .iter()
        .any(|command| command.text.as_deref() == Some("Checking responsive rules")));
    assert!(text_commands
        .iter()
        .all(|command| command.text_layout_policy == HostTextLayoutPolicy::WordWrap));
    assert!(text_commands
        .iter()
        .all(|command| command.frame.width > 0.0));
}

#[test]
fn agent_chat_falls_back_to_node_text_when_message_model_is_empty() {
    let node = TemplatePaneNodeData {
        text: "A single assistant response".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 120.0,
        height: 80.0,
    };
    let mut commands = Vec::new();

    push_agent_chat(&mut commands, &node, &rect, &rect, 0, 1.0);

    assert!(commands
        .iter()
        .any(|command| command.text.as_deref() == Some("A single assistant response")));
}

#[test]
fn agent_chat_keeps_a_full_thread_as_individual_role_aligned_bubbles() {
    let node = TemplatePaneNodeData {
        collection_items: messages(&[
            "user|First question",
            "agent|First answer",
            "user|Follow-up with more context",
            "agent|Second answer with a tool result",
        ]),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 320.0,
        height: 180.0,
    };
    let mut commands = Vec::new();

    push_agent_chat(&mut commands, &node, &rect, &rect, 10, 1.0);

    let text_commands = commands
        .iter()
        .filter(|command| command.text.is_some())
        .collect::<Vec<_>>();
    assert_eq!(text_commands.len(), 4);
    assert!(text_commands
        .iter()
        .all(|command| command.frame.width > 0.0 && command.frame.height > 0.0));
    assert!(text_commands
        .iter()
        .all(|command| command.frame.x >= rect.x && command.frame.right() <= rect.right()));
    assert!(text_commands
        .windows(2)
        .all(|pair| pair[0].frame.y < pair[1].frame.y));
    assert!(text_commands[0].frame.x > text_commands[1].frame.x);
    assert!(text_commands[2].frame.x > text_commands[3].frame.x);
}
