use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::first_non_empty;
use super::geometry::alert_message_frame;
use super::style::alert_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

// 命令序列在表面与操作之间绘制正文；取首个非空文本候选，并采用运行时换行策略。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_message(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    left: f32,
    right: f32,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let message = alert_message(node);
    if message.is_empty() || right <= left {
        return;
    }
    let Some((frame, font_size, line_height)) = alert_message_frame(node, rect, left, right) else {
        return;
    };
    commands.push(HostPaintCommand::wrapped_text(
        frame,
        Some(clip.clone()),
        order,
        message.to_string(),
        alert_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn alert_message(node: &TemplatePaneNodeData) -> &str {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.validation_message.as_str(),
        node.options_text.as_str(),
    ])
}

#[cfg(test)]
#[path = "tests/message.rs"]
mod tests;

#[cfg(test)]
#[path = "message/tests/capacity_tests.rs"]
mod capacity_tests;
