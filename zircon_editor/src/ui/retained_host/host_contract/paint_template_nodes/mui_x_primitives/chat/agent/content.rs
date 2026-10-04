use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::super::paint_theme::current_host_metrics;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::style::chat_text_color;
use super::messages::agent_chat_message_layout;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const CHAT_TEXT_INSET_X: f32 = 8.0;
const CHAT_TEXT_INSET_Y: f32 = 5.0;
const CHAT_TEXT_BOTTOM_INSET: f32 = 5.0;

pub(super) fn push_agent_chat_content(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let messages = agent_chat_message_layout(node, rect);
    if messages.is_empty() {
        return;
    }

    let metrics = current_host_metrics();
    let font_size = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        metrics.font_small
    };
    let line_height = metrics.line_height(font_size).max(font_size);
    let color = chat_text_color(node);

    for message in messages {
        let frame = message.frame;
        let text = message.text;
        let text_frame = FrameRect {
            x: frame.x + CHAT_TEXT_INSET_X,
            y: frame.y + CHAT_TEXT_INSET_Y,
            width: frame.width - CHAT_TEXT_INSET_X * 2.0,
            height: frame.height - CHAT_TEXT_INSET_Y - CHAT_TEXT_BOTTOM_INSET,
        };
        if text_frame.width <= 0.0 || text_frame.height < line_height {
            continue;
        }
        commands.push(HostPaintCommand::wrapped_text(
            text_frame,
            Some(clip.clone()),
            order,
            text,
            color,
            font_size,
            line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}

#[cfg(test)]
#[path = "tests/content.rs"]
mod tests;
