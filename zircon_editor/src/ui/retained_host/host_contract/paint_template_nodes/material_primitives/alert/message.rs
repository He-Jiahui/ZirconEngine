use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::first_non_empty;
use super::geometry::alert_message_frame;
use super::style::alert_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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
        message,
        alert_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn alert_message(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.validation_message.as_str(),
        node.options_text.as_str(),
    ])
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;

    #[test]
    fn tall_alert_message_uses_runtime_word_wrap() {
        let node = TemplatePaneNodeData {
            text: "Asset import needs validation before opening the selected project.".into(),
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 260.0,
            height: 88.0,
        };
        let mut commands = Vec::new();

        push_alert_message(&mut commands, &node, &rect, 28.0, 254.0, &rect, 2, 1.0);

        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0].text_layout_policy,
            HostTextLayoutPolicy::WordWrap
        );
        assert!(commands[0].frame.height > commands[0].line_height);
    }
}
