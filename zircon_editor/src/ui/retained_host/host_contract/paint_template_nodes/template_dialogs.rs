use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

mod actions;
mod style;

const DIALOG_PADDING_X: f32 = 20.0;
const DIALOG_TITLE_TOP: f32 = 18.0;
const DIALOG_BODY_TOP: f32 = 48.0;
const DIALOG_TITLE_FONT_SIZE: f32 = 15.0;
const DIALOG_TITLE_LINE_HEIGHT: f32 = 18.0;
const DIALOG_BODY_FONT_SIZE: f32 = 12.5;
const DIALOG_BODY_LINE_HEIGHT: f32 = 16.0;
const DIALOG_CORNER_RADIUS: f32 = 6.0;
const DIALOG_BORDER_WIDTH: f32 = 1.0;
const CONFIRM_SEVERITY_MARK_WIDTH: f32 = 4.0;

pub(super) fn push_dialog_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = dialog_kind(node) else {
        return false;
    };
    if !node.popup_open {
        return true;
    }

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 1.0 || rect.height <= 1.0 {
        return true;
    }

    let unavailable = style::dialog_unavailable(node);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style::dialog_surface_color(unavailable)),
        Some(style::dialog_border_color(node, kind, unavailable)),
        DIALOG_BORDER_WIDTH,
        DIALOG_CORNER_RADIUS,
        opacity,
    ));

    if matches!(kind, DialogKind::ConfirmDialog) {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x,
                y: rect.y,
                width: CONFIRM_SEVERITY_MARK_WIDTH,
                height: rect.height,
            },
            Some(clip.clone()),
            order + 1,
            Some(style::severity_mark_color(node)),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }

    let content_left = rect.x + DIALOG_PADDING_X;
    let content_width = (rect.width - DIALOG_PADDING_X * 2.0).max(1.0);
    let title = title_text(node);
    if let Some(title) = title {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: content_left,
                y: rect.y + DIALOG_TITLE_TOP,
                width: content_width,
                height: DIALOG_TITLE_LINE_HEIGHT,
            },
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
            FrameRect {
                x: content_left,
                y: rect.y + DIALOG_BODY_TOP,
                width: content_width,
                height: DIALOG_BODY_LINE_HEIGHT,
            },
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

    actions::push_dialog_actions(
        commands,
        node,
        &rect,
        clip,
        order,
        kind,
        unavailable,
        opacity,
    );
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogKind {
    Dialog,
    ConfirmDialog,
}

fn dialog_kind(node: &TemplatePaneNodeData) -> Option<DialogKind> {
    match (node.role.as_str(), node.component_role.as_str()) {
        ("Dialog", _) | (_, "dialog") => Some(DialogKind::Dialog),
        ("ConfirmDialog", _) | (_, "confirm-dialog") | ("AlertDialog", _) | (_, "alert-dialog") => {
            Some(DialogKind::ConfirmDialog)
        }
        _ => None,
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

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
