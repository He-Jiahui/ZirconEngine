use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::DialogKind;
use super::{layout, metrics::dialog_metrics, style};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dialog_content(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: DialogKind,
    unavailable: bool,
    action_top: Option<f32>,
    opacity: f32,
) {
    let metrics = dialog_metrics();
    let title = title_text(node);
    if let (Some(title), Some(title_frame)) = (title, layout::title_rect(rect)) {
        if layout::frame_is_within(clip, &title_frame) {
            commands.push(HostPaintCommand::text(
                title_frame,
                Some(clip.clone()),
                order + 2,
                title.to_string(),
                style::dialog_title_color(node, kind, unavailable),
                metrics.title_font_size,
                metrics.title_line_height,
                UiTextRunPaintStyle::default(),
                opacity,
            ));
        }
    }

    if let (Some(message), Some(body)) = (
        message_text(node, title),
        layout::body_rect(rect, kind, action_top),
    ) {
        if layout::frame_is_within(clip, &body) {
            commands.push(HostPaintCommand::text(
                body,
                Some(clip.clone()),
                order + 3,
                message.to_string(),
                style::dialog_body_color(unavailable),
                metrics.body_font_size,
                metrics.body_line_height,
                UiTextRunPaintStyle::default(),
                opacity,
            ));
        }
    }
}

fn title_text(node: &TemplatePaneNodeData) -> Option<&str> {
    first_non_empty(&[node.text.as_str(), node.label_text.as_str()])
}

fn message_text<'a>(node: &'a TemplatePaneNodeData, title: Option<&str>) -> Option<&'a str> {
    let message = first_non_empty(&[node.value_text.as_str(), node.validation_message.as_str()])?;
    if title.is_some_and(|title| title == message) {
        None
    } else {
        Some(message)
    }
}

fn first_non_empty<'a>(values: &[&'a str]) -> Option<&'a str> {
    values.iter().copied().find_map(non_empty)
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}
