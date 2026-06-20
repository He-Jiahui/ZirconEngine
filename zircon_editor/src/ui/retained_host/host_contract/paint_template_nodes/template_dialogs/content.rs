use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::identity::DialogKind;
use super::{layout, style};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DIALOG_TITLE_FONT_SIZE: f32 = 15.0;
const DIALOG_TITLE_LINE_HEIGHT: f32 = 18.0;
const DIALOG_BODY_FONT_SIZE: f32 = 12.5;
const DIALOG_BODY_LINE_HEIGHT: f32 = 16.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dialog_content(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: DialogKind,
    unavailable: bool,
    opacity: f32,
) {
    let title = title_text(node);
    if let Some(title) = title {
        commands.push(HostPaintCommand::text(
            layout::title_rect(rect),
            Some(clip.clone()),
            order + 2,
            title.to_string(),
            style::dialog_title_color(node, kind, unavailable),
            DIALOG_TITLE_FONT_SIZE,
            DIALOG_TITLE_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    if let Some(message) = message_text(node, title) {
        commands.push(HostPaintCommand::text(
            layout::body_rect(rect),
            Some(clip.clone()),
            order + 3,
            message.to_string(),
            style::dialog_body_color(unavailable),
            DIALOG_BODY_FONT_SIZE,
            DIALOG_BODY_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
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
