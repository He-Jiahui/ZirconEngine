use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;

const MUI_X_CHAT_INSET: f32 = 6.0;
const MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION: f32 = 0.24;
const MUI_X_CHAT_STREAMING_HEIGHT: f32 = 3.0;

#[derive(Clone, Copy)]
pub(super) enum ChatKind {
    AgentChat,
    Composer,
}

pub(super) fn chat_kind(component_role: &str, role: &str) -> Option<ChatKind> {
    if super::matches_any_role(component_role, role, &["mui-x-agent-chat", "AgentChat"]) {
        Some(ChatKind::AgentChat)
    } else if super::matches_any_role(
        component_role,
        role,
        &["mui-x-chat-composer", "ChatComposer"],
    ) {
        Some(ChatKind::Composer)
    } else {
        None
    }
}

pub(super) fn push_chat(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ChatKind,
) {
    match kind {
        ChatKind::AgentChat => push_agent_chat(commands, node, rect, clip, order, opacity),
        ChatKind::Composer => push_chat_composer(commands, node, rect, clip, order, opacity),
    }
}

fn push_agent_chat(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::node_radius(node).max(8.0);
    super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::node_background(node).unwrap_or_else(|| chat_surface_color(node)),
        0.0,
        radius,
        opacity,
    );

    let bubble_height = (rect.height * MUI_X_CHAT_BUBBLE_HEIGHT_FRACTION).max(8.0);
    super::push_quad(
        commands,
        FrameRect {
            x: rect.x + MUI_X_CHAT_INSET,
            y: rect.y + MUI_X_CHAT_INSET,
            width: rect.width * 0.58,
            height: bubble_height,
        },
        clip,
        order + 1,
        PALETTE.surface,
        0.0,
        5.0,
        opacity,
    );
    super::push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.36,
            y: rect.y + MUI_X_CHAT_INSET + bubble_height + 3.0,
            width: (rect.width * 0.58 - MUI_X_CHAT_INSET).max(1.0),
            height: bubble_height,
        },
        clip,
        order + 2,
        PALETTE.surface_selected,
        0.0,
        5.0,
        opacity,
    );

    if node.component_variant.as_str().contains("streaming") || node.popup_open {
        super::push_quad(
            commands,
            FrameRect {
                x: rect.x + MUI_X_CHAT_INSET,
                y: rect.y + rect.height - MUI_X_CHAT_INSET,
                width: (rect.width * 0.42).max(1.0),
                height: MUI_X_CHAT_STREAMING_HEIGHT,
            },
            clip,
            order + 3,
            PALETTE.accent,
            0.0,
            MUI_X_CHAT_STREAMING_HEIGHT * 0.5,
            opacity,
        );
    }
}

fn push_chat_composer(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::node_radius(node).max(rect.height * 0.5);
    super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::node_background(node).unwrap_or(PALETTE.surface_inset),
        1.0,
        radius,
        opacity,
    );
    super::push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width - rect.height + 4.0,
            y: rect.y + 4.0,
            width: (rect.height - 8.0).max(1.0),
            height: (rect.height - 8.0).max(1.0),
        },
        clip,
        order + 1,
        PALETTE.accent,
        0.0,
        rect.height,
        opacity,
    );
}

fn chat_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error_container
    } else if node.component_variant.as_str().contains("streaming") {
        PALETTE.info_container
    } else {
        PALETTE.surface_inset
    }
}
