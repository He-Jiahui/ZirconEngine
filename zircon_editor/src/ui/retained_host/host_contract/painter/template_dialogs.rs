use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DIALOG_PADDING_X: f32 = 20.0;
const DIALOG_TITLE_TOP: f32 = 18.0;
const DIALOG_BODY_TOP: f32 = 48.0;
const DIALOG_ACTION_BOTTOM: f32 = 20.0;
const DIALOG_ACTION_GAP: f32 = 16.0;
const DIALOG_ACTION_MIN_WIDTH: f32 = 56.0;
const DIALOG_ACTION_CHAR_WIDTH: f32 = 7.0;
const DIALOG_TITLE_FONT_SIZE: f32 = 15.0;
const DIALOG_TITLE_LINE_HEIGHT: f32 = 18.0;
const DIALOG_BODY_FONT_SIZE: f32 = 12.5;
const DIALOG_BODY_LINE_HEIGHT: f32 = 16.0;
const DIALOG_ACTION_FONT_SIZE: f32 = 12.5;
const DIALOG_ACTION_LINE_HEIGHT: f32 = 16.0;
const DIALOG_CORNER_RADIUS: f32 = 6.0;
const DIALOG_BORDER_WIDTH: f32 = 1.0;
const CONFIRM_SEVERITY_MARK_WIDTH: f32 = 4.0;

const DIALOG_SURFACE: [u8; 4] = [23, 28, 32, 255];
const DIALOG_BORDER: [u8; 4] = [52, 63, 71, 255];
const DIALOG_ACTIVE_BORDER: [u8; 4] = [53, 199, 208, 255];
const DIALOG_TITLE: [u8; 4] = [232, 236, 238, 255];
const DIALOG_BODY: [u8; 4] = [164, 174, 180, 255];
const DIALOG_ACTION: [u8; 4] = [53, 199, 208, 255];
const DIALOG_INFO: [u8; 4] = [53, 199, 208, 255];
const DIALOG_INFO_BORDER: [u8; 4] = [41, 101, 150, 255];
const DIALOG_WARNING: [u8; 4] = [224, 163, 58, 255];
const DIALOG_WARNING_BORDER: [u8; 4] = [132, 94, 35, 255];
const DIALOG_ERROR: [u8; 4] = [239, 112, 102, 255];
const DIALOG_ERROR_BORDER: [u8; 4] = [133, 61, 58, 255];
const DIALOG_DISABLED_SURFACE: [u8; 4] = [37, 44, 49, 255];
const DIALOG_DISABLED_BORDER: [u8; 4] = [52, 63, 71, 255];
const DIALOG_DISABLED_TEXT: [u8; 4] = [89, 101, 108, 255];

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

    let unavailable = node.disabled || variant_contains_any(node, &["disabled", "loading"]);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(dialog_surface_color(unavailable)),
        Some(dialog_border_color(node, kind, unavailable)),
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
            Some(severity_mark_color(node)),
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
            dialog_title_color(node, kind, unavailable),
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
            dialog_body_color(unavailable),
            DIALOG_BODY_FONT_SIZE,
            DIALOG_BODY_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    push_dialog_actions(
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogSeverity {
    Info,
    Warning,
    Error,
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

fn push_dialog_actions(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: DialogKind,
    unavailable: bool,
    opacity: f32,
) {
    let action_y = rect.y + rect.height - DIALOG_ACTION_BOTTOM - DIALOG_ACTION_LINE_HEIGHT;
    let mut action_right = rect.x + rect.width - DIALOG_PADDING_X;
    if matches!(kind, DialogKind::ConfirmDialog) {
        let confirm = action_label(node, 1).unwrap_or_else(|| "Confirm".to_string());
        let confirm_width = action_width(&confirm);
        let confirm_enabled = confirm_enabled(node) && !unavailable;
        action_right -= confirm_width;
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: action_right,
                y: action_y,
                width: confirm_width,
                height: DIALOG_ACTION_LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 5,
            confirm,
            confirm_action_color(node, unavailable, confirm_enabled),
            DIALOG_ACTION_FONT_SIZE,
            DIALOG_ACTION_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
        action_right -= DIALOG_ACTION_GAP;

        let cancel = action_label(node, 0).unwrap_or_else(|| "Cancel".to_string());
        let cancel_width = action_width(&cancel);
        action_right -= cancel_width;
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: action_right,
                y: action_y,
                width: cancel_width,
                height: DIALOG_ACTION_LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 4,
            cancel,
            cancel_action_color(unavailable),
            DIALOG_ACTION_FONT_SIZE,
            DIALOG_ACTION_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
        return;
    }

    let Some(action) = action_label(node, 0) else {
        return;
    };
    let width = action_width(&action);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: action_right - width,
            y: action_y,
            width,
            height: DIALOG_ACTION_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order + 4,
        action,
        dialog_action_color(unavailable),
        DIALOG_ACTION_FONT_SIZE,
        DIALOG_ACTION_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
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

fn action_label(node: &TemplatePaneNodeData, index: usize) -> Option<String> {
    node.actions
        .row_data(index)
        .and_then(|action| non_empty(action.label.as_str()).map(str::to_string))
}

fn dialog_surface_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_SURFACE
    } else {
        DIALOG_SURFACE
    }
}

fn dialog_border_color(
    node: &TemplatePaneNodeData,
    kind: DialogKind,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_BORDER
    } else if matches!(kind, DialogKind::ConfirmDialog) {
        severity_border_color(node)
    } else if node.focused || node.pressed || node.popup_open {
        DIALOG_ACTIVE_BORDER
    } else {
        DIALOG_BORDER
    }
}

fn dialog_title_color(node: &TemplatePaneNodeData, kind: DialogKind, unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else if matches!(kind, DialogKind::ConfirmDialog)
        && (variant_contains_any(node, &["destructive"])
            || matches!(severity(node), DialogSeverity::Error))
    {
        severity_mark_color(node)
    } else {
        DIALOG_TITLE
    }
}

fn dialog_body_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_BODY
    }
}

fn dialog_action_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_ACTION
    }
}

fn cancel_action_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_BODY
    }
}

fn confirm_action_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    confirm_enabled: bool,
) -> [u8; 4] {
    if unavailable || !confirm_enabled {
        DIALOG_DISABLED_TEXT
    } else if variant_contains_any(node, &["destructive"]) {
        DIALOG_ERROR
    } else {
        DIALOG_ACTION
    }
}

fn confirm_enabled(node: &TemplatePaneNodeData) -> bool {
    !variant_contains_any(
        node,
        &[
            "confirmDisabled",
            "confirm-disabled",
            "confirm_disabled",
            "disabledConfirm",
        ],
    )
}

fn severity(node: &TemplatePaneNodeData) -> DialogSeverity {
    if variant_contains_any(node, &["info"]) {
        DialogSeverity::Info
    } else if variant_contains_any(node, &["error", "danger"]) {
        DialogSeverity::Error
    } else {
        DialogSeverity::Warning
    }
}

fn severity_mark_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match severity(node) {
        DialogSeverity::Info => DIALOG_INFO,
        DialogSeverity::Warning => DIALOG_WARNING,
        DialogSeverity::Error => DIALOG_ERROR,
    }
}

fn severity_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match severity(node) {
        DialogSeverity::Info => DIALOG_INFO_BORDER,
        DialogSeverity::Warning => DIALOG_WARNING_BORDER,
        DialogSeverity::Error => DIALOG_ERROR_BORDER,
    }
}

fn action_width(text: &str) -> f32 {
    (text.chars().count() as f32 * DIALOG_ACTION_CHAR_WIDTH + 20.0).max(DIALOG_ACTION_MIN_WIDTH)
}

fn variant_contains_any(node: &TemplatePaneNodeData, expected: &[&str]) -> bool {
    [
        node.component_variant.as_str(),
        node.surface_variant.as_str(),
        node.validation_level.as_str(),
        node.text_tone.as_str(),
        node.button_variant.as_str(),
    ]
    .iter()
    .flat_map(|value| value.split_whitespace())
    .any(|part| {
        expected
            .iter()
            .any(|expected| part.eq_ignore_ascii_case(expected))
    })
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
