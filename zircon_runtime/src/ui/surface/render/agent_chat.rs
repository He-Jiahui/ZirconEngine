use crate::ui::icon_atlas::{builtin_icon_supported, UiIconSize};
use toml::Value;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    style::UiPainterResolvedState,
    surface::{
        UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiRichTextFormat, UiVisualAssetRef,
    },
    tree::UiTemplateNodeMetadata,
};

use super::clipping::intersect_clip_frame;

#[cfg(test)]
use super::resolve::resolve_style;

const CHAT_INSET: f32 = 6.0;
const CHAT_THREAD_GAP: f32 = 4.0;
const CHAT_THREAD_ASSISTANT_WIDTH_FRACTION: f32 = 0.72;
const CHAT_THREAD_USER_BUBBLE_WIDTH_FRACTION: f32 = 0.56;
const CHAT_THREAD_MAX_VISIBLE_MESSAGES: usize = 12;
const CHAT_TEXT_INSET_X: f32 = 8.0;
const CHAT_TEXT_INSET_Y: f32 = 5.0;
const CHAT_TEXT_BOTTOM_INSET: f32 = 5.0;
const CHAT_COMPOSER_TEXT_INSET_X: f32 = 8.0;
const CHAT_COMPOSER_TEXT_INSET_Y: f32 = 4.0;
const CHAT_COMPOSER_TEXT_RIGHT_RESERVE: f32 = 10.0;
const CHAT_COMPOSER_SEND_INSET: f32 = 4.0;
const CHAT_COMPOSER_ICON_INSET: f32 = 4.0;
const DEFAULT_SEND_ICON: &str = "arrow-up";

const FALLBACK_AGENT_BUBBLE: &str = "#1d2330";
const FALLBACK_USER_BUBBLE: &str = "#26344a";
const FALLBACK_TEXT: &str = "#f4f7fb";
const FALLBACK_MUTED_TEXT: &str = "#718096";
const FALLBACK_ACCENT: &str = "#8b7cff";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AgentMessageRole {
    Assistant,
    User,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AgentMessage {
    role: AgentMessageRole,
    text: String,
}

#[derive(Clone, Debug, PartialEq)]
struct AgentMessageLayout {
    role: AgentMessageRole,
    text: String,
    frame: UiFrame,
}

/// Returns the runtime-owned paint commands for the ReactBits/MUI X chat primitives.
///
/// The node's owner command remains responsible for the authored panel surface. This module
/// adds the semantic message rows, optional user/assistant bubbles, text, streaming affordance,
/// composer value, and send action so the same ZUI source can be rendered by the shared runtime
/// and the retained host.
/// The small `role|text` transport is deliberately mirrored by the native host contract.
pub(super) fn agent_chat_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if !valid_frame(frame) {
        return Vec::new();
    }

    if agent_chat_component(metadata) {
        return render_agent_chat(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        );
    }
    if chat_composer_component(metadata) {
        return render_chat_composer(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        );
    }
    Vec::new()
}

/// AgentChat and ChatComposer own their semantic text payloads. Suppressing
/// the generic owner label prevents a duplicated fallback text command when a
/// node also carries `text` for accessibility or authoring metadata.
pub(super) fn agent_chat_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata
        .is_some_and(|metadata| agent_chat_component(metadata) || chat_composer_component(metadata))
}

fn agent_chat_component(metadata: &UiTemplateNodeMetadata) -> bool {
    component_matches(metadata, &["AgentChat", "mui-x-agent-chat"])
}

fn chat_composer_component(metadata: &UiTemplateNodeMetadata) -> bool {
    component_matches(metadata, &["ChatComposer", "mui-x-chat-composer"])
}

fn component_matches(metadata: &UiTemplateNodeMetadata, candidates: &[&str]) -> bool {
    let normalize = |value: &str| {
        value
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    let values = std::iter::once(metadata.component.as_str()).chain(
        metadata
            .attributes
            .get("component_role")
            .and_then(Value::as_str),
    );
    values.map(normalize).any(|value| {
        candidates
            .iter()
            .any(|candidate| normalize(candidate) == value)
    })
}

fn render_agent_chat(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let messages = agent_messages(metadata);
    if messages.is_empty() {
        return streaming_indicator_command(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        )
        .into_iter()
        .collect();
    }

    let layouts = agent_message_layout(
        metadata,
        messages,
        frame,
        message_minimum_height(base_style),
        streaming_indicator_reserve(metadata, base_style),
    );
    let mut commands = Vec::with_capacity(layouts.len().saturating_mul(2) + 1);
    let bubble_radius = metric_attribute(metadata, "bubble_corner_radius")
        .or_else(|| metric_attribute(metadata, "message_radius"))
        .or_else(|| metric_attribute(metadata, "bubble_radius"))
        .or_else(|| (base_style.corner_radius > 0.0).then_some(base_style.corner_radius))
        .unwrap_or(0.0)
        .max(0.0);
    let bubble_border = color_attribute(
        metadata,
        &[
            "bubble_border_color",
            "message_border_color",
            "bubble_outline",
        ],
    );
    let bubble_border_width = metric_attribute(metadata, "bubble_border_width")
        .or_else(|| metric_attribute(metadata, "message_border_width"))
        .unwrap_or(0.0)
        .max(0.0);
    let bubble_text_color = color_attribute(metadata, &["message_text_color", "bubble_text_color"])
        .or_else(|| base_style.foreground_color.clone())
        .unwrap_or_else(|| FALLBACK_TEXT.to_string());
    let disabled = bool_attribute(metadata, "disabled").unwrap_or(false)
        || matches!(
            base_style.painter_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        );
    let bubble_text_color = disabled
        .then(|| {
            color_attribute(
                metadata,
                &["disabled_foreground_color", "disabled_text_color"],
            )
            .unwrap_or_else(|| FALLBACK_MUTED_TEXT.to_string())
        })
        .unwrap_or(bubble_text_color);

    let assistant_bubble = bool_attribute(metadata, "assistant_bubble")
        .or_else(|| bool_attribute(metadata, "assistant_message_bubble"))
        .unwrap_or(false);
    for (index, message) in layouts.into_iter().enumerate() {
        let bubble_color = match message.role {
            AgentMessageRole::Assistant => color_attribute(
                metadata,
                &[
                    "agent_bubble_color",
                    "assistant_bubble_color",
                    "agent_message_background_color",
                    "assistant_message_background_color",
                ],
            )
            .unwrap_or_else(|| FALLBACK_AGENT_BUBBLE.to_string()),
            AgentMessageRole::User => color_attribute(
                metadata,
                &[
                    "user_bubble_color",
                    "user_message_background_color",
                    "human_bubble_color",
                ],
            )
            .unwrap_or_else(|| FALLBACK_USER_BUBBLE.to_string()),
        };
        let bubble_z = z_index.saturating_add(1 + index as i32 * 2);
        // ReactBits AI Chat keeps assistant replies in the conversation's
        // continuous reading flow and gives the user turn the right-aligned
        // bubble.  A caller can opt into assistant bubbles for a different
        // product variant, but the reference-semantic default is unadorned
        // assistant prose.
        if message.role == AgentMessageRole::User || assistant_bubble {
            commands.push(quad_command(
                node_id,
                message.frame,
                clip_frame,
                bubble_z,
                bubble_color,
                bubble_border.clone(),
                bubble_border_width,
                bubble_radius,
                base_style,
                opacity,
            ));
        }

        let text_frame = UiFrame::new(
            message.frame.x + CHAT_TEXT_INSET_X,
            message.frame.y + CHAT_TEXT_INSET_Y,
            (message.frame.width - CHAT_TEXT_INSET_X * 2.0).max(0.0),
            (message.frame.height - CHAT_TEXT_INSET_Y - CHAT_TEXT_BOTTOM_INSET).max(0.0),
        );
        if valid_frame(text_frame) {
            commands.push(text_command(
                node_id,
                text_frame,
                intersect_clip_frame(clip_frame, text_frame),
                bubble_z.saturating_add(1),
                message.text,
                bubble_text_color.clone(),
                base_style,
                opacity,
            ));
        }
    }

    if let Some(indicator) = streaming_indicator_command(
        node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
    ) {
        commands.push(indicator);
    }
    commands
}

fn render_chat_composer(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let mut commands = Vec::with_capacity(3);
    let text = composer_text(metadata);
    let font_size = metric_attribute(metadata, "font_size")
        .or_else(|| (base_style.font_size > 0.0).then_some(base_style.font_size))
        .unwrap_or(UiResolvedStyle::DEFAULT_FONT_SIZE)
        .max(1.0);
    let line_height = metric_attribute(metadata, "line_height")
        .or_else(|| metric_attribute(metadata, "line_height_ratio").map(|ratio| font_size * ratio))
        .unwrap_or_else(|| base_style.line_height.max(font_size));
    let send_extent = (frame.height - CHAT_COMPOSER_SEND_INSET * 2.0).max(1.0);
    let right_reserve = send_extent + CHAT_COMPOSER_TEXT_RIGHT_RESERVE;
    let text_frame = UiFrame::new(
        frame.x + CHAT_COMPOSER_TEXT_INSET_X,
        frame.y + ((frame.height - line_height) * 0.5).max(CHAT_COMPOSER_TEXT_INSET_Y),
        (frame.width - right_reserve - CHAT_COMPOSER_TEXT_INSET_X * 2.0).max(0.0),
        line_height.min(frame.height.max(0.0)),
    );
    if let Some(text) = text.filter(|text| !text.trim().is_empty()) {
        if valid_frame(text_frame) && text_frame.bottom() <= frame.bottom() + f32::EPSILON {
            let color = if bool_attribute(metadata, "disabled").unwrap_or(false) {
                color_attribute(
                    metadata,
                    &["disabled_foreground_color", "disabled_text_color"],
                )
                .unwrap_or_else(|| FALLBACK_MUTED_TEXT.to_string())
            } else {
                color_attribute(metadata, &["composer_text_color", "text_color"])
                    .or_else(|| base_style.foreground_color.clone())
                    .unwrap_or_else(|| FALLBACK_TEXT.to_string())
            };
            commands.push(text_command(
                node_id,
                text_frame,
                intersect_clip_frame(clip_frame, text_frame),
                z_index.saturating_add(1),
                text,
                color,
                base_style,
                opacity,
            ));
        }
    }

    let send_color = color_attribute(
        metadata,
        &[
            "send_color",
            "accent_color",
            "streaming_color",
            "progress_color",
        ],
    )
    .or_else(|| base_style.border_color.clone())
    .unwrap_or_else(|| FALLBACK_ACCENT.to_string());
    let send_frame = UiFrame::new(
        frame.right() - send_extent - CHAT_COMPOSER_SEND_INSET,
        frame.y + CHAT_COMPOSER_SEND_INSET,
        send_extent,
        send_extent,
    );
    if valid_frame(send_frame) {
        commands.push(quad_command(
            node_id,
            send_frame,
            clip_frame,
            z_index.saturating_add(2),
            send_color,
            None,
            0.0,
            frame.height.max(1.0),
            base_style,
            opacity,
        ));

        if let Some(icon) = send_icon_reference(metadata) {
            let icon_frame = UiFrame::new(
                send_frame.x + CHAT_COMPOSER_ICON_INSET,
                send_frame.y + CHAT_COMPOSER_ICON_INSET,
                (send_frame.width - CHAT_COMPOSER_ICON_INSET * 2.0).max(1.0),
                (send_frame.height - CHAT_COMPOSER_ICON_INSET * 2.0).max(1.0),
            );
            if valid_frame(icon_frame) {
                commands.push(icon_command(
                    node_id,
                    icon_frame,
                    clip_frame,
                    z_index.saturating_add(3),
                    icon,
                    color_attribute(metadata, &["send_icon_color", "icon_color"])
                        .or_else(|| base_style.foreground_color.clone())
                        .unwrap_or_else(|| FALLBACK_TEXT.to_string()),
                    base_style,
                    opacity,
                ));
            }
        }
    }
    commands
}

fn send_icon_reference(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    let authored_icon = string_attribute(metadata, "send_icon")
        .or_else(|| string_attribute(metadata, "icon"))
        .unwrap_or(DEFAULT_SEND_ICON)
        .trim();
    if authored_icon.is_empty() {
        return None;
    }
    if authored_icon.contains('@') {
        return builtin_icon_supported(authored_icon).then(|| authored_icon.to_string());
    }
    if !builtin_icon_supported(authored_icon) {
        return None;
    }
    let tier = match metadata.attributes.get("send_icon_size") {
        None => UiIconSize::Small,
        Some(_) => UiIconSize::parse(string_attribute(metadata, "send_icon_size")?)?,
    };
    Some(format!("{authored_icon}@{}", tier.as_str()))
}

fn agent_messages(metadata: &UiTemplateNodeMetadata) -> Vec<AgentMessage> {
    let mut parsed = Vec::new();
    for key in ["messages", "collection_items", "items"] {
        if let Some(value) = metadata.attributes.get(key) {
            collect_message_values(value, &mut parsed);
            if !parsed.is_empty() {
                break;
            }
        }
    }
    if parsed.is_empty() {
        if let Some(text) = string_attribute(metadata, "text") {
            parsed.extend(
                text.lines()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| parse_message_text(value, None)),
            );
        }
    }

    parsed
        .into_iter()
        .enumerate()
        .filter_map(|(index, (role, text))| {
            let text = text.trim();
            (!text.is_empty()).then(|| AgentMessage {
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

fn collect_message_values(value: &Value, output: &mut Vec<(Option<AgentMessageRole>, String)>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_message_values(value, output);
            }
        }
        Value::String(value) => output.push(parse_message_text(value, None)),
        Value::Table(table) => {
            let role = ["role", "sender", "author", "type"]
                .iter()
                .filter_map(|key| table.get(*key).and_then(Value::as_str))
                .find_map(parse_role);
            let text = [
                "text", "content", "message", "body", "value", "label", "title",
            ]
            .iter()
            .find_map(|key| table.get(*key).and_then(scalar_text));
            if let Some(text) = text {
                output.push((role, text));
            }
        }
        _ => {}
    }
}

fn parse_message_text(
    value: &str,
    explicit_role: Option<AgentMessageRole>,
) -> (Option<AgentMessageRole>, String) {
    if let Some(role) = explicit_role {
        return (Some(role), value.trim().to_string());
    }
    for separator in ['|', ':'] {
        if let Some((prefix, body)) = value.split_once(separator) {
            if let Some(role) = parse_role(prefix) {
                return (Some(role), body.trim().to_string());
            }
        }
    }
    (None, value.trim().to_string())
}

fn parse_role(value: &str) -> Option<AgentMessageRole> {
    match value.trim().to_ascii_lowercase().as_str() {
        "assistant" | "agent" | "system" | "tool" => Some(AgentMessageRole::Assistant),
        "user" | "human" | "you" => Some(AgentMessageRole::User),
        _ => None,
    }
}

fn agent_message_layout(
    metadata: &UiTemplateNodeMetadata,
    messages: Vec<AgentMessage>,
    frame: UiFrame,
    minimum_message_height: f32,
    indicator_reserve: f32,
) -> Vec<AgentMessageLayout> {
    let max_visible = usize_attribute(metadata, "max_visible_messages")
        .unwrap_or(CHAT_THREAD_MAX_VISIBLE_MESSAGES)
        .clamp(1, 64);
    let total_count = messages.len();
    let visible_count = total_count.min(max_visible);
    let mut visible = messages.into_iter().take(visible_count).collect::<Vec<_>>();
    if visible_count < total_count {
        let overflow_text = string_attribute(metadata, "overflow_text")
            .map(str::trim)
            .filter(|text| !text.is_empty());
        if let (Some(last), Some(overflow_text)) = (visible.last_mut(), overflow_text) {
            last.text.push('\n');
            last.text.push_str(overflow_text);
        }
    }

    let inset = metric_attribute(metadata, "chat_inset")
        .or_else(|| metric_attribute(metadata, "message_inset"))
        .unwrap_or(CHAT_INSET)
        .max(0.0);
    let gap = metric_attribute(metadata, "thread_gap")
        .unwrap_or(CHAT_THREAD_GAP)
        .max(0.0);
    let count = visible.len() as f32;
    let inner_height = (frame.height - inset * 2.0 - indicator_reserve).max(0.0);
    let gap = if visible.len() > 1 && inner_height >= count * minimum_message_height {
        let max_gap = (inner_height - count * minimum_message_height) / (count - 1.0);
        gap.min(max_gap.max(0.0))
    } else {
        0.0
    };
    let bubble_height = ((inner_height - gap * (count - 1.0)) / count).max(0.0);
    let max_width = (frame.width - inset * 2.0).max(1.0);

    visible
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            let default_width = match message.role {
                AgentMessageRole::Assistant => CHAT_THREAD_ASSISTANT_WIDTH_FRACTION,
                AgentMessageRole::User => CHAT_THREAD_USER_BUBBLE_WIDTH_FRACTION,
            };
            let width_fraction = metric_attribute(
                metadata,
                match message.role {
                    AgentMessageRole::Assistant => "assistant_width_fraction",
                    AgentMessageRole::User => "user_bubble_width_fraction",
                },
            )
            .or_else(|| metric_attribute(metadata, "bubble_width_fraction"))
            .unwrap_or(default_width)
            .clamp(0.05, 1.0);
            let bubble_width = (frame.width * width_fraction).min(max_width).max(1.0);
            let x = match message.role {
                AgentMessageRole::Assistant => frame.x + inset,
                AgentMessageRole::User => frame.right() - inset - bubble_width,
            };
            AgentMessageLayout {
                role: message.role,
                text: message.text,
                frame: UiFrame::new(
                    x,
                    frame.y + inset + index as f32 * (bubble_height + gap),
                    bubble_width,
                    bubble_height,
                ),
            }
        })
        .collect()
}

fn streaming_indicator_command(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Option<UiRenderCommand> {
    if !chat_streaming_active(metadata) {
        return None;
    }
    let inset = metric_attribute(metadata, "chat_inset")
        .or_else(|| metric_attribute(metadata, "message_inset"))
        .unwrap_or(CHAT_INSET)
        .max(0.0);
    let indicator_height = streaming_indicator_height(metadata, base_style);
    let indicator_width_fraction = metric_attribute(metadata, "streaming_indicator_width_fraction")
        .unwrap_or(0.42)
        .clamp(0.05, 1.0);
    let indicator_frame = UiFrame::new(
        frame.x + inset,
        frame.bottom() - inset - indicator_height,
        (frame.width * indicator_width_fraction).max(1.0),
        indicator_height.min(frame.height.max(0.0)),
    );
    valid_frame(indicator_frame).then(|| {
        quad_command(
            node_id,
            indicator_frame,
            clip_frame,
            z_index.saturating_add(5),
            color_attribute(
                metadata,
                &["streaming_color", "accent_color", "progress_color"],
            )
            .or_else(|| base_style.border_color.clone())
            .unwrap_or_else(|| FALLBACK_ACCENT.to_string()),
            None,
            0.0,
            indicator_height * 0.5,
            base_style,
            opacity,
        )
    })
}

fn message_minimum_height(base_style: &UiResolvedStyle) -> f32 {
    base_style.line_height.max(base_style.font_size).max(1.0)
        + CHAT_TEXT_INSET_Y
        + CHAT_TEXT_BOTTOM_INSET
}

fn streaming_indicator_height(
    metadata: &UiTemplateNodeMetadata,
    base_style: &UiResolvedStyle,
) -> f32 {
    metric_attribute(metadata, "streaming_indicator_height")
        .unwrap_or_else(|| (base_style.font_size * 0.25).clamp(1.0, 4.0))
        .max(1.0)
}

fn streaming_indicator_reserve(
    metadata: &UiTemplateNodeMetadata,
    base_style: &UiResolvedStyle,
) -> f32 {
    chat_streaming_active(metadata)
        .then(|| {
            metric_attribute(metadata, "streaming_indicator_reserve")
                .unwrap_or_else(|| {
                    streaming_indicator_height(metadata, base_style) + CHAT_THREAD_GAP
                })
                .max(0.0)
        })
        .unwrap_or(0.0)
}

fn chat_streaming_active(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "streaming").unwrap_or(false)
        || bool_attribute(metadata, "popup_open").unwrap_or(false)
        || metadata.classes.iter().any(|class| {
            class.eq_ignore_ascii_case("streaming")
                || class.to_ascii_lowercase().contains("streaming")
        })
        || ["component_variant", "variant", "mui_variant"]
            .iter()
            .filter_map(|key| string_attribute(metadata, key))
            .any(|variant| variant.to_ascii_lowercase().contains("streaming"))
}

fn composer_text(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    [
        "composer_text",
        "composerText",
        "value_text",
        "value",
        "text",
    ]
    .iter()
    .find_map(|key| metadata.attributes.get(*key).and_then(scalar_text))
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
}

fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background_color: String,
    border_color: Option<String>,
    border_width: f32,
    corner_radius: f32,
    base_style: &UiResolvedStyle,
    opacity: f32,
) -> UiRenderCommand {
    let mut style = base_style.clone();
    style.background_color = Some(background_color);
    style.border_color = border_color;
    style.border_width = border_width;
    style.corner_radius = corner_radius;
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style,
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

fn icon_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    icon: String,
    foreground_color: String,
    base_style: &UiResolvedStyle,
    opacity: f32,
) -> UiRenderCommand {
    let mut style = base_style.clone();
    style.background_color = None;
    style.border_color = None;
    style.border_width = 0.0;
    style.corner_radius = 0.0;
    style.foreground_color = Some(foreground_color);
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Image,
        frame,
        clip_frame,
        z_index,
        style,
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon)),
        opacity,
    }
}

fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground_color: String,
    base_style: &UiResolvedStyle,
    opacity: f32,
) -> UiRenderCommand {
    let mut style = base_style.clone();
    style.background_color = None;
    style.border_color = None;
    style.border_width = 0.0;
    style.corner_radius = 0.0;
    style.foreground_color = Some(foreground_color);
    style.wrap = zircon_runtime_interface::ui::surface::UiTextWrap::Word;
    // A message is one localized reading flow. Inline emphasis (for example
    // `**important**`) must resolve as runs inside this command rather than
    // becoming adjacent labels that can reorder or wrap independently.
    style.rich_text_format = UiRichTextFormat::MarkdownInlineV1;
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style,
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}

fn color_attribute(metadata: &UiTemplateNodeMetadata, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        metadata
            .style_overrides
            .get(*key)
            .or_else(|| metadata.attributes.get(*key))
            .and_then(color_value)
    })
}

fn color_value(value: &Value) -> Option<String> {
    match value {
        Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
        Value::Table(table) => table.get("color").and_then(color_value),
        _ => None,
    }
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Integer(value) => Some(value.to_string()),
        Value::Float(value) => Some(value.to_string()),
        Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(|value| match value {
        Value::Boolean(value) => Some(*value),
        Value::String(value) => match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        },
        _ => None,
    })
}

fn metric_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(|value| match value {
            Value::Integer(value) => Some(*value as f32),
            Value::Float(value) if value.is_finite() => Some(*value as f32),
            _ => None,
        })
        .filter(|value| value.is_finite())
}

fn usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    metric_attribute(metadata, key).and_then(|value| (value >= 0.0).then_some(value as usize))
}

fn valid_frame(frame: UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

#[cfg(test)]
#[path = "tests/agent_chat.rs"]
mod tests;
