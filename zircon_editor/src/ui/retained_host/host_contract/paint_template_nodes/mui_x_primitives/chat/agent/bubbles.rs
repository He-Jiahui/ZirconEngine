use super::super::super::super::super::data::FrameRect;
use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::super::render_commands::HostPaintCommand;
use super::messages::{agent_chat_message_layout, AgentMessageRole};

type AgentBubbleColors = [[u8; 4]; 2];

pub(super) fn push_agent_bubbles(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let [agent_color, user_color] = agent_bubble_colors_from_host(current_host_palette());
    for (index, message) in agent_chat_message_layout(node, rect)
        .into_iter()
        .enumerate()
    {
        let color = match message.role {
            AgentMessageRole::Assistant => agent_color,
            AgentMessageRole::User => user_color,
        };
        super::super::super::push_quad(
            commands,
            message.frame,
            clip,
            order + index as i32,
            color,
            0.0,
            5.0,
            opacity,
        );
    }
}

fn agent_bubble_colors_from_host(palette: HostMaterialPalette) -> AgentBubbleColors {
    [palette.surface, palette.surface_selected]
}

#[cfg(test)]
#[path = "tests/bubbles.rs"]
mod tests;
