use std::collections::BTreeMap;

use super::*;

fn metadata(component: &str, attributes: &[(&str, Value)]) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        attributes: attributes
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect::<BTreeMap<_, _>>(),
        ..UiTemplateNodeMetadata::default()
    }
}

#[test]
fn agent_chat_emits_source_ordered_text_and_role_aligned_user_turns() {
    let node = metadata(
        "AgentChat",
        &[
            (
                "messages",
                Value::Array(
                    [
                        "user|Question",
                        "agent|Answer",
                        "user|Follow-up",
                        "agent|Second answer",
                    ]
                    .into_iter()
                    .map(|value| Value::String(value.to_string()))
                    .collect(),
                ),
            ),
            ("streaming", Value::Boolean(true)),
        ],
    );
    let style = resolve_style(Some(&node));
    let commands = agent_chat_render_commands(
        UiNodeId::new(1),
        Some(&node),
        UiFrame::new(4.0, 6.0, 320.0, 180.0),
        None,
        10,
        1.0,
        &style,
    );

    let text = commands
        .iter()
        .filter(|command| command.kind == UiRenderCommandKind::Text)
        .collect::<Vec<_>>();
    assert_eq!(text.len(), 4);
    assert_eq!(text[0].text.as_deref(), Some("Question"));
    assert_eq!(text[1].text.as_deref(), Some("Answer"));
    assert!(text
        .windows(2)
        .all(|pair| pair[0].frame.y < pair[1].frame.y));
    assert!(text[0].frame.x > text[1].frame.x);
    assert!(text[2].frame.x > text[3].frame.x);
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.kind == UiRenderCommandKind::Quad)
            .count(),
        3,
        "two user bubbles plus the streaming affordance; assistant prose stays in flow"
    );
}

#[test]
fn short_thread_preserves_source_order_and_keeps_assistant_copy_in_flow() {
    let node = metadata(
        "AgentChat",
        &[(
            "messages",
            Value::Array(vec![
                Value::String("user|Question".to_string()),
                Value::String("agent|Answer with **one** emphasized phrase".to_string()),
            ]),
        )],
    );
    let style = resolve_style(Some(&node));
    let commands = agent_chat_render_commands(
        UiNodeId::new(6),
        Some(&node),
        UiFrame::new(0.0, 0.0, 320.0, 120.0),
        None,
        0,
        1.0,
        &style,
    );
    let text = commands
        .iter()
        .filter(|command| command.kind == UiRenderCommandKind::Text)
        .collect::<Vec<_>>();
    assert_eq!(text.len(), 2);
    assert_eq!(text[0].text.as_deref(), Some("Question"));
    assert_eq!(
        text[1].text.as_deref(),
        Some("Answer with **one** emphasized phrase")
    );
    assert!(text[0].frame.y < text[1].frame.y);
    assert!(text[0].frame.x > text[1].frame.x);
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.kind == UiRenderCommandKind::Quad)
            .count(),
        1,
        "the short thread has one user bubble; assistant prose stays unboxed"
    );
}

#[test]
fn composer_emits_value_text_before_the_send_action() {
    let node = metadata(
        "ChatComposer",
        &[(
            "composer_text",
            Value::String("Continue the parity review".to_string()),
        )],
    );
    let style = resolve_style(Some(&node));
    let commands = agent_chat_render_commands(
        UiNodeId::new(2),
        Some(&node),
        UiFrame::new(4.0, 8.0, 220.0, 44.0),
        None,
        2,
        1.0,
        &style,
    );
    let text_index = commands
        .iter()
        .position(|command| command.kind == UiRenderCommandKind::Text)
        .expect("composer text");
    let send_index = commands
        .iter()
        .position(|command| command.kind == UiRenderCommandKind::Quad)
        .expect("send action");
    assert_eq!(
        commands[text_index].text.as_deref(),
        Some("Continue the parity review")
    );
    assert!(text_index < send_index);
    let icon = commands
        .iter()
        .find(|command| command.kind == UiRenderCommandKind::Image)
        .expect("semantic send icon");
    assert_eq!(
        icon.image,
        Some(UiVisualAssetRef::Icon("arrow-up@s".to_string()))
    );
    assert!(icon.frame.x >= commands[send_index].frame.x);
    assert!(icon.frame.bottom() <= commands[send_index].frame.bottom());
    assert!(commands[send_index].frame.right() <= 224.0);
}

#[test]
fn composer_rejects_numeric_icon_sizes_instead_of_inventing_a_tier() {
    let node = metadata(
        "ChatComposer",
        &[
            ("composer_text", Value::String("Prompt".to_string())),
            ("send_icon_size", Value::String("18".to_string())),
        ],
    );
    let style = resolve_style(Some(&node));
    let commands = agent_chat_render_commands(
        UiNodeId::new(8),
        Some(&node),
        UiFrame::new(0.0, 0.0, 220.0, 44.0),
        None,
        0,
        1.0,
        &style,
    );

    assert!(commands
        .iter()
        .all(|command| command.kind != UiRenderCommandKind::Image));
}

#[test]
fn message_keeps_inline_emphasis_in_one_rich_text_command() {
    let node = metadata(
        "AgentChat",
        &[(
            "messages",
            Value::Array(vec![Value::String(
                "agent|The **important** result stays in this sentence.".to_string(),
            )]),
        )],
    );
    let style = resolve_style(Some(&node));
    let commands = agent_chat_render_commands(
        UiNodeId::new(5),
        Some(&node),
        UiFrame::new(0.0, 0.0, 320.0, 120.0),
        None,
        0,
        1.0,
        &style,
    );

    let text = commands
        .iter()
        .filter(|command| command.kind == UiRenderCommandKind::Text)
        .collect::<Vec<_>>();
    assert_eq!(text.len(), 1);
    assert_eq!(
        text[0].text.as_deref(),
        Some("The **important** result stays in this sentence.")
    );
    assert_eq!(
        text[0].style.rich_text_format,
        UiRichTextFormat::MarkdownInlineV1
    );
}

#[test]
fn table_messages_and_unprefixed_values_are_supported() {
    let node = metadata(
        "AgentChat",
        &[(
            "messages",
            Value::Array(vec![
                Value::Table(
                    [
                        ("role".to_string(), Value::String("user".into())),
                        ("content".to_string(), Value::String("Question".into())),
                    ]
                    .into_iter()
                    .collect(),
                ),
                Value::String("Answer".into()),
            ]),
        )],
    );
    let messages = agent_messages(&node);
    assert_eq!(messages[0].role, AgentMessageRole::User);
    assert_eq!(messages[1].role, AgentMessageRole::User);
    assert_eq!(messages[1].text, "Answer");
}

#[test]
fn overflow_copy_stays_caller_owned() {
    let messages = [
        "agent|First answer",
        "user|Second question",
        "agent|Third answer",
        "user|Fourth question",
    ]
    .into_iter()
    .map(|value| Value::String(value.to_string()))
    .collect();
    let node = metadata(
        "AgentChat",
        &[
            ("messages", Value::Array(messages)),
            ("max_visible_messages", Value::Integer(2)),
        ],
    );
    let style = resolve_style(Some(&node));
    let commands = agent_chat_render_commands(
        UiNodeId::new(3),
        Some(&node),
        UiFrame::new(0.0, 0.0, 320.0, 180.0),
        None,
        0,
        1.0,
        &style,
    );

    let text = commands
        .iter()
        .filter(|command| command.kind == UiRenderCommandKind::Text)
        .collect::<Vec<_>>();
    assert_eq!(text.len(), 2);
    assert_eq!(text[0].text.as_deref(), Some("First answer"));
    assert_eq!(text[1].text.as_deref(), Some("Second question"));
}

#[test]
fn caller_owned_overflow_copy_stays_in_the_last_visible_text_flow() {
    let messages = [
        "agent|First answer",
        "user|Second question",
        "agent|Third answer",
        "user|Fourth question",
    ]
    .into_iter()
    .map(|value| Value::String(value.to_string()))
    .collect();
    let node = metadata(
        "AgentChat",
        &[
            ("messages", Value::Array(messages)),
            ("max_visible_messages", Value::Integer(2)),
            (
                "overflow_text",
                Value::String("Two more localized messages".to_string()),
            ),
        ],
    );
    let style = resolve_style(Some(&node));
    let commands = agent_chat_render_commands(
        UiNodeId::new(4),
        Some(&node),
        UiFrame::new(0.0, 0.0, 320.0, 180.0),
        None,
        0,
        1.0,
        &style,
    );

    let text = commands
        .iter()
        .filter(|command| command.kind == UiRenderCommandKind::Text)
        .collect::<Vec<_>>();
    assert_eq!(text.len(), 2);
    assert_eq!(
        text[1].text.as_deref(),
        Some("Second question\nTwo more localized messages")
    );
}

#[test]
fn semantic_chat_components_suppress_generic_owner_text_and_accept_role_aliases() {
    let chat = metadata(
        "Panel",
        &[(
            "component_role",
            Value::String("mui-x-agent-chat".to_string()),
        )],
    );
    let composer = metadata("mui-x-chat-composer", &[]);
    let ordinary = metadata("Panel", &[]);

    assert!(agent_chat_suppresses_owner_text(Some(&chat)));
    assert!(agent_chat_suppresses_owner_text(Some(&composer)));
    assert!(!agent_chat_suppresses_owner_text(Some(&ordinary)));
    assert!(chat_composer_component(&composer));
}

#[test]
fn streaming_variant_reserves_the_same_indicator_lane_as_streaming_state() {
    let node = metadata(
        "AgentChat",
        &[(
            "component_variant",
            Value::String("compact-streaming".to_string()),
        )],
    );
    assert!(chat_streaming_active(&node));
    assert!(streaming_indicator_reserve(&node, &UiResolvedStyle::default()) > 0.0);
}

#[test]
fn assistant_bubble_is_opt_in_for_product_variants() {
    let node = metadata(
        "AgentChat",
        &[
            (
                "messages",
                Value::Array(vec![Value::String("agent|Answer".to_string())]),
            ),
            ("assistant_bubble", Value::Boolean(true)),
        ],
    );
    let style = resolve_style(Some(&node));
    let commands = agent_chat_render_commands(
        UiNodeId::new(7),
        Some(&node),
        UiFrame::new(0.0, 0.0, 320.0, 120.0),
        None,
        0,
        1.0,
        &style,
    );
    assert_eq!(
        commands
            .iter()
            .filter(|command| command.kind == UiRenderCommandKind::Quad)
            .count(),
        1
    );
}
