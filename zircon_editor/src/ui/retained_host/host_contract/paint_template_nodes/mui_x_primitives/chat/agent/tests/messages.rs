use super::*;
use crate::ui::retained_host::primitives::{ModelRc, SharedString, VecModel};
use std::rc::Rc;

fn messages(values: &[&str]) -> ModelRc<SharedString> {
    ModelRc::from(Rc::new(VecModel::from(
        values
            .iter()
            .map(|value| SharedString::from(*value))
            .collect::<Vec<_>>(),
    )))
}

#[test]
fn full_thread_layout_preserves_source_order_and_role_alignment() {
    let node = TemplatePaneNodeData {
        collection_items: messages(&[
            "user|Question",
            "agent|Answer",
            "user|Follow-up",
            "agent|Second answer",
        ]),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 2.0,
        y: 3.0,
        width: 300.0,
        height: 180.0,
    };

    let layout = agent_chat_message_layout(&node, &rect);

    assert_eq!(layout.len(), 4);
    assert_eq!(layout[0].text, "Question");
    assert_eq!(layout[1].text, "Answer");
    assert!(layout
        .windows(2)
        .all(|pair| pair[0].frame.y < pair[1].frame.y));
    assert!(layout[0].frame.x > layout[1].frame.x);
    assert!(layout[2].frame.x > layout[3].frame.x);
    assert!(layout
        .iter()
        .all(|item| item.frame.x >= rect.x && item.frame.right() <= rect.right()));
}

#[test]
fn legacy_layout_keeps_assistant_before_user_for_two_messages() {
    let node = TemplatePaneNodeData {
        collection_items: messages(&["user|Question", "agent|Answer"]),
        ..TemplatePaneNodeData::default()
    };
    let layout = agent_chat_message_layout(
        &node,
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 160.0,
        },
    );

    assert_eq!(
        layout
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>(),
        ["Answer", "Question"]
    );
}

#[test]
fn dense_thread_layout_stays_inside_card_bounds() {
    let values = (0..THREAD_MAX_VISIBLE_MESSAGES)
        .map(|index| {
            if index % 2 == 0 {
                format!("user|Message {index}")
            } else {
                format!("agent|Message {index}")
            }
        })
        .collect::<Vec<_>>();
    let value_refs = values.iter().map(String::as_str).collect::<Vec<_>>();
    let node = TemplatePaneNodeData {
        collection_items: messages(&value_refs),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 5.0,
        y: 7.0,
        width: 180.0,
        height: 64.0,
    };

    let layout = agent_chat_message_layout(&node, &rect);

    assert_eq!(layout.len(), THREAD_MAX_VISIBLE_MESSAGES);
    assert!(layout.iter().all(|item| {
        item.frame.x >= rect.x
            && item.frame.y >= rect.y
            && item.frame.right() <= rect.right()
            && item.frame.bottom() <= rect.bottom()
    }));
}
