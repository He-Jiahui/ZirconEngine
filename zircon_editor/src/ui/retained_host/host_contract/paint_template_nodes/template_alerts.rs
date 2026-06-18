use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    select_workbench_alert_style, select_workbench_toast_style, WorkbenchAlertTone as AlertTone,
};
use super::template_alert_glyphs::{push_alert_mark, push_close_mark, ALERT_ICON_SIZE};
use super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const ALERT_FONT_SIZE: f32 = 12.0;
const ALERT_LINE_HEIGHT: f32 = ALERT_FONT_SIZE * 1.2;
const ALERT_RADIUS: f32 = 4.0;
const ALERT_BORDER_WIDTH: f32 = 1.0;
const ALERT_ICON_LEFT: f32 = 10.0;
const ALERT_TEXT_GAP: f32 = 8.0;
const ALERT_TEXT_RIGHT_INSET: f32 = 10.0;

const TOAST_FONT_SIZE: f32 = 11.5;
const TOAST_LINE_HEIGHT: f32 = TOAST_FONT_SIZE * 1.25;
const TOAST_RADIUS: f32 = 5.0;
const TOAST_ICON_LEFT: f32 = 12.0;
const TOAST_ICON_SIZE: f32 = 18.0;
const TOAST_TEXT_GAP: f32 = 9.0;
const TOAST_TRAILING_INSET: f32 = 10.0;
const TOAST_CLOSE_SIZE: f32 = 14.0;
const TOAST_ACTION_WIDTH: f32 = 44.0;
const TOAST_ACTION_TEXT: &str = "UNDO";

pub(super) fn push_alert_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = workbench_alert_kind(node) else {
        return false;
    };
    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    match kind {
        WorkbenchAlertKind::Inline(tone) => {
            push_inline_alert(commands, node, &rect, clip, order, tone, opacity);
        }
        WorkbenchAlertKind::Toast => {
            push_toast(commands, node, &rect, clip, order, opacity);
        }
    }
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkbenchAlertKind {
    Inline(AlertTone),
    Toast,
}

fn workbench_alert_kind(node: &TemplatePaneNodeData) -> Option<WorkbenchAlertKind> {
    match node.control_id.as_str() {
        "WorkbenchInfoAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Info)),
        "WorkbenchSuccessAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Success)),
        "WorkbenchWarningAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Warning)),
        "WorkbenchErrorAlert" => Some(WorkbenchAlertKind::Inline(AlertTone::Error)),
        "WorkbenchToastRoot" if is_standalone_toast(node) => Some(WorkbenchAlertKind::Toast),
        "WorkbenchToastRoot" => Some(WorkbenchAlertKind::Inline(
            alert_tone(node).unwrap_or(AlertTone::Info),
        )),
        _ if node.control_id.as_str().starts_with("Workbench")
            && (matches!(node.role.as_str(), "Alert")
                || matches!(node.component_role.as_str(), "alert" | "mui-alert")
                || node.control_id.as_str().ends_with("Alert")) =>
        {
            alert_tone(node).map(WorkbenchAlertKind::Inline)
        }
        _ => None,
    }
}

fn is_standalone_toast(node: &TemplatePaneNodeData) -> bool {
    let label = template_node_label(node, None).to_ascii_lowercase();
    label.contains("operation completed") || label.contains("completed successfully")
}

fn alert_tone(node: &TemplatePaneNodeData) -> Option<AlertTone> {
    let key = format!(
        "{} {} {} {} {} {}",
        node.control_id.as_str(),
        node.icon_name.as_str(),
        node.validation_level.as_str(),
        node.text_tone.as_str(),
        node.component_variant.as_str(),
        template_node_label(node, None)
    )
    .to_ascii_lowercase();
    if key.contains("warning") {
        Some(AlertTone::Warning)
    } else if key.contains("error") || key.contains("danger") || key.contains("failed") {
        Some(AlertTone::Error)
    } else if key.contains("success") || key.contains("check") {
        Some(AlertTone::Success)
    } else if key.contains("info") {
        Some(AlertTone::Info)
    } else {
        None
    }
}

fn push_inline_alert(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tone: AlertTone,
    opacity: f32,
) {
    let style = select_workbench_alert_style(node, tone);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        ALERT_BORDER_WIDTH,
        ALERT_RADIUS,
        opacity,
    ));

    let icon = FrameRect {
        x: rect.x + ALERT_ICON_LEFT,
        y: rect.y + (rect.height - ALERT_ICON_SIZE).max(0.0) * 0.5,
        width: ALERT_ICON_SIZE,
        height: ALERT_ICON_SIZE,
    };
    push_alert_mark(commands, &icon, clip, order + 1, tone, style.mark, opacity);

    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let text_left = icon.x + icon.width + ALERT_TEXT_GAP;
    let text_right = rect.x + rect.width - ALERT_TEXT_RIGHT_INSET;
    if text_right <= text_left {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: text_left,
            y: rect.y + (rect.height - ALERT_LINE_HEIGHT).max(0.0) * 0.5,
            width: text_right - text_left,
            height: ALERT_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order + 2,
        label,
        style.text,
        ALERT_FONT_SIZE,
        ALERT_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_toast(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = select_workbench_toast_style(node);
    let icon_size = toast_status_mark_size(node);

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        1.0,
        TOAST_RADIUS,
        opacity,
    ));

    let icon = FrameRect {
        x: rect.x + TOAST_ICON_LEFT,
        y: rect.y + (rect.height - icon_size).max(0.0) * 0.5,
        width: icon_size,
        height: icon_size,
    };
    push_alert_mark(
        commands,
        &icon,
        clip,
        order + 1,
        AlertTone::Success,
        style.mark,
        opacity,
    );

    let has_action = rect.width >= 210.0;
    let close = toast_close_rect(rect);
    let action_left = close.x - TOAST_ACTION_WIDTH;
    let text_right = if has_action {
        action_left - 4.0
    } else {
        rect.x + rect.width - TOAST_TRAILING_INSET
    };
    let text_left = icon.x + icon.width + TOAST_TEXT_GAP;
    let label = template_node_label(node, None);
    if !label.trim().is_empty() && text_right > text_left {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: text_left,
                y: rect.y + (rect.height - TOAST_LINE_HEIGHT).max(0.0) * 0.5,
                width: text_right - text_left,
                height: TOAST_LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 2,
            label,
            style.text,
            TOAST_FONT_SIZE,
            TOAST_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    if has_action {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: action_left,
                y: rect.y + (rect.height - TOAST_LINE_HEIGHT).max(0.0) * 0.5,
                width: TOAST_ACTION_WIDTH,
                height: TOAST_LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 2,
            TOAST_ACTION_TEXT.to_string(),
            style.action,
            TOAST_FONT_SIZE,
            TOAST_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
        push_close_mark(commands, &close, clip, order + 3, style.close, opacity);
    }
}

fn toast_status_mark_size(node: &TemplatePaneNodeData) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        TOAST_ICON_SIZE
    }
}

fn toast_close_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - TOAST_TRAILING_INSET - TOAST_CLOSE_SIZE,
        y: rect.y + (rect.height - TOAST_CLOSE_SIZE).max(0.0) * 0.5,
        width: TOAST_CLOSE_SIZE,
        height: TOAST_CLOSE_SIZE,
    }
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

#[cfg(test)]
#[path = "template_alerts_tests.rs"]
mod tests;
