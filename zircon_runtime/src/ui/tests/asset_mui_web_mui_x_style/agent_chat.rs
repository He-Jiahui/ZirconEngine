use super::*;

#[test]
fn mui_x_agent_chat_utility_classes_match_retained_targets() {
    let style = UiAssetLoader::load_toml_str(MUI_X_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_X_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_asset(style)
        .expect("style asset registration succeeds");
    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let chat = find_node(root, "AgentChatRoot");
    assert_eq!(str_attr(chat, "validation_level"), Some("agent-chat-error"));
    assert_eq!(str_attr(chat, "text_tone"), Some("agent-chat-with-content"));
    assert_classes(
        chat,
        &[
            "MuiAgentChat-root",
            "MuiAgentChat-streaming",
            "MuiAgentChat-error",
            "MuiAgentChat-hasMessages",
            "MuiAgentChat-hasComposerText",
        ],
    );

    let messages = find_node(root, "AgentChatMessages");
    assert_eq!(
        str_attr(messages, "surface_variant"),
        Some("agent-chat-messages-state")
    );
    assert_classes(
        messages,
        &[
            "MuiAgentChat-messages",
            "MuiAgentChat-messagesPopulated",
            "MuiAgentChat-messagesStreaming",
            "MuiAgentChat-messagesError",
        ],
    );

    let composer = find_node(root, "AgentChatComposer");
    assert_eq!(
        str_attr(composer, "text_tone"),
        Some("agent-chat-composer-state")
    );
    assert_classes(
        composer,
        &[
            "MuiAgentChat-composer",
            "MuiAgentChat-composerStreaming",
            "MuiAgentChat-composerError",
            "MuiAgentChat-composerHasText",
            "composer-extra",
        ],
    );

    let conversation_list = find_node(root, "ChatConversationListRoot");
    assert_eq!(
        str_attr(conversation_list, "surface_variant"),
        Some("chat-conversation-list-populated")
    );
    assert_classes(
        conversation_list,
        &[
            "MuiChatConversationList-root",
            "MuiChatConversationList-populated",
        ],
    );

    let message_list = find_node(root, "ChatMessageListRoot");
    assert_eq!(
        str_attr(message_list, "surface_variant"),
        Some("chat-message-list-populated")
    );
    assert_classes(
        message_list,
        &["MuiChatMessageList-root", "MuiChatMessageList-populated"],
    );

    let message = find_node(root, "ChatMessageListMessage");
    assert_eq!(
        str_attr(message, "text_tone"),
        Some("chat-message-populated")
    );
    assert_classes(
        message,
        &["MuiChatMessage-root", "MuiChatMessage-populated"],
    );

    let chat_composer = find_node(root, "ChatComposerRoot");
    assert_eq!(
        str_attr(chat_composer, "text_tone"),
        Some("chat-composer-streaming")
    );
    assert_eq!(
        str_attr(chat_composer, "validation_level"),
        Some("chat-composer-active")
    );
    assert_classes(
        chat_composer,
        &[
            "MuiChatComposer-root",
            "MuiChatComposer-streaming",
            "MuiChatComposer-hasText",
        ],
    );
}
