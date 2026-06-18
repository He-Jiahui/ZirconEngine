use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::{style, DialogKind, DIALOG_PADDING_X};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DIALOG_ACTION_BOTTOM: f32 = 20.0;
const DIALOG_ACTION_GAP: f32 = 16.0;
const DIALOG_ACTION_MIN_WIDTH: f32 = 56.0;
const DIALOG_ACTION_CHAR_WIDTH: f32 = 7.0;
const DIALOG_ACTION_FONT_SIZE: f32 = 12.5;
const DIALOG_ACTION_LINE_HEIGHT: f32 = 16.0;

pub(super) fn push_dialog_actions(
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
        let confirm_enabled = style::confirm_enabled(node) && !unavailable;
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
            style::confirm_action_color(node, unavailable, confirm_enabled),
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
            style::cancel_action_color(unavailable),
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
        style::dialog_action_color(unavailable),
        DIALOG_ACTION_FONT_SIZE,
        DIALOG_ACTION_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn action_label(node: &TemplatePaneNodeData, index: usize) -> Option<String> {
    node.actions
        .row_data(index)
        .and_then(|action| non_empty(action.label.as_str()).map(str::to_string))
}

fn action_width(text: &str) -> f32 {
    (text.chars().count() as f32 * DIALOG_ACTION_CHAR_WIDTH + 20.0).max(DIALOG_ACTION_MIN_WIDTH)
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}
