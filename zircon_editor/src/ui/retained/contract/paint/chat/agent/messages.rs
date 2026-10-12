use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::metrics::{MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION, MUI_X_CHAT_INSET};

const THREAD_GAP: f32 = 4.0;
const THREAD_INDICATOR_RESERVE: f32 = 8.0;
const THREAD_BUBBLE_WIDTH_FRACTION: f32 = 0.72;
const THREAD_MIN_BUBBLE_HEIGHT: f32 = 16.0;
const THREAD_MAX_VISIBLE_MESSAGES: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AgentMessageRole {
    Assistant,
    User,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AgentChatMessage {
    pub(super) role: AgentMessageRole,
    pub(super) text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct AgentChatMessageLayout {
    pub(super) role: AgentMessageRole,
    pub(super) text: String,
    pub(super) frame: FrameRect,
}

/// Project the authored message transport into ordered, role-aware messages.
///
/// `role|text` and `role:text` are intentionally small transport formats so
/// the retained host can preserve the source-owned collection without adding
/// a second message model. Unprefixed values alternate from assistant to user,
/// matching the legacy two-slot painter's first-available-slot behavior.
pub(super) fn agent_chat_messages(node: &TemplatePaneNodeData) -> Vec<AgentChatMessage> {
    let mut values = node
        .collection_items
        .iter()
        .map(|value| value.as_str().trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() && !node.text.trim().is_empty() {
        values = node
            .text
            .split('\n')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect();
    }

    values
        .into_iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let (role, text) = parse_message_value(&value);
            let text = text.trim();
            if text.is_empty() {
                return None;
            }
            Some(AgentChatMessage {
                role: role.unwrap_or_else(|| {
                    if index % 2 == 0 {
                        AgentMessageRole::Assistant
                    } else {
                        AgentMessageRole::User
                    }
                }),
                text: text.to_string(),
            })
        })
        .collect()
}

/// Build bubble frames for both the legacy two-slot card and a full thread.
///
/// Two-message cards retain the established geometry for visual stability.
/// Once a source owns more than two messages, each message receives its own
/// row and role-aligned bubble, which lets AI Chat/Tool detail compositions
/// remain readable without baking copy into fixed coordinates.
pub(super) fn agent_chat_message_layout(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> Vec<AgentChatMessageLayout> {
    let messages = agent_chat_messages(node);
    if messages.is_empty() {
        return Vec::new();
    }
    if messages.len() <= 2 {
        return legacy_message_layout(messages, rect);
    }

    let total_count = messages.len();
    let visible_count = total_count.min(THREAD_MAX_VISIBLE_MESSAGES);
    let mut visible = messages.into_iter().take(visible_count).collect::<Vec<_>>();
    if visible_count < total_count {
        let remaining = total_count - visible_count;
        if let Some(last) = visible.last_mut() {
            last.text.push_str(&format!("\n… +{remaining} more"));
        }
    }

    let count = visible.len() as f32;
    let inset = MUI_X_CHAT_INSET;
    let inner_height = (rect.height - inset * 2.0 - THREAD_INDICATOR_RESERVE).max(0.0);
    // Preserve the readable minimum whenever the card has room for it. When
    // a compact surface cannot fit every row at that minimum, collapse the
    // gaps first and proportionally share the remaining height instead of
    // letting the last bubbles escape the card's clip bounds.
    let gap = if visible.len() > 1 && inner_height >= count * THREAD_MIN_BUBBLE_HEIGHT {
        let max_gap_for_minimum = (inner_height - count * THREAD_MIN_BUBBLE_HEIGHT) / (count - 1.0);
        THREAD_GAP.min(max_gap_for_minimum.max(0.0))
    } else {
        0.0
    };
    let bubble_height = ((inner_height - gap * (count - 1.0)) / count).max(0.0);
    let max_width = (rect.width - inset * 2.0).max(1.0);
    let bubble_width = (rect.width * THREAD_BUBBLE_WIDTH_FRACTION)
        .min(max_width)
        .max(1.0);

    visible
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            let x = match message.role {
                AgentMessageRole::Assistant => rect.x + inset,
                AgentMessageRole::User => rect.right() - inset - bubble_width,
            };
            AgentChatMessageLayout {
                role: message.role,
                text: message.text,
                frame: FrameRect {
                    x,
                    y: rect.y + inset + index as f32 * (bubble_height + gap),
                    width: bubble_width,
                    height: bubble_height,
                },
            }
        })
        .collect()
}

fn legacy_message_layout(
    messages: Vec<AgentChatMessage>,
    rect: &FrameRect,
) -> Vec<AgentChatMessageLayout> {
    let [assistant_frame, user_frame] = legacy_bubble_frames(rect);
    let mut slots: [Option<AgentChatMessage>; 2] = [None, None];
    for message in messages {
        let preferred = role_slot(message.role);
        let slot = if slots[preferred].is_none() {
            preferred
        } else if slots[1 - preferred].is_none() {
            1 - preferred
        } else {
            preferred
        };
        if let Some(existing) = slots[slot].as_mut() {
            existing.text.push('\n');
            existing.text.push_str(&message.text);
        } else {
            slots[slot] = Some(message);
        }
    }

    [
        (slots[0].take(), assistant_frame),
        (slots[1].take(), user_frame),
    ]
    .into_iter()
    .filter_map(|(message, frame)| {
        message.map(|message| AgentChatMessageLayout {
            role: message.role,
            text: message.text,
            frame,
        })
    })
    .collect()
}

fn role_slot(role: AgentMessageRole) -> usize {
    match role {
        AgentMessageRole::Assistant => 0,
        AgentMessageRole::User => 1,
    }
}

fn legacy_bubble_frames(rect: &FrameRect) -> [FrameRect; 2] {
    let bubble_height = (rect.height * MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION).max(8.0);
    [
        FrameRect {
            x: rect.x + MUI_X_CHAT_INSET,
            y: rect.y + MUI_X_CHAT_INSET,
            width: (rect.width * 0.58).max(1.0),
            height: bubble_height,
        },
        FrameRect {
            x: rect.x + rect.width * 0.36,
            y: rect.y + MUI_X_CHAT_INSET + bubble_height + 3.0,
            width: (rect.width * 0.58 - MUI_X_CHAT_INSET).max(1.0),
            height: bubble_height,
        },
    ]
}

fn parse_message_value(value: &str) -> (Option<AgentMessageRole>, &str) {
    for separator in ['|', ':'] {
        let Some((prefix, body)) = value.split_once(separator) else {
            continue;
        };
        let role = match prefix.trim().to_ascii_lowercase().as_str() {
            "assistant" | "agent" | "system" => Some(AgentMessageRole::Assistant),
            "user" | "human" | "you" => Some(AgentMessageRole::User),
            _ => None,
        };
        if role.is_some() && !body.trim().is_empty() {
            return (role, body.trim());
        }
    }
    (None, value.trim())
}

#[cfg(test)]
#[path = "tests/messages.rs"]
mod tests;
