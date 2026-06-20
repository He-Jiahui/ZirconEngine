use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::identity::DialogKind;
use super::super::{layout, style};
use super::labels::{action_label, action_width};
use super::metrics::{DIALOG_ACTION_BOTTOM, DIALOG_ACTION_GAP, DIALOG_ACTION_LINE_HEIGHT};
use super::text::push_dialog_action_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dialog_actions(
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
    let mut action_right = layout::action_right(rect);
    if matches!(kind, DialogKind::ConfirmDialog) {
        let confirm = action_label(node, 1).unwrap_or_else(|| "Confirm".to_string());
        let confirm_width = action_width(&confirm);
        let confirm_enabled = style::confirm_enabled(node) && !unavailable;
        action_right -= confirm_width;
        push_dialog_action_text(
            commands,
            FrameRect {
                x: action_right,
                y: action_y,
                width: confirm_width,
                height: DIALOG_ACTION_LINE_HEIGHT,
            },
            clip,
            order + 5,
            confirm,
            style::confirm_action_color(node, unavailable, confirm_enabled),
            opacity,
        );
        action_right -= DIALOG_ACTION_GAP;

        let cancel = action_label(node, 0).unwrap_or_else(|| "Cancel".to_string());
        let cancel_width = action_width(&cancel);
        action_right -= cancel_width;
        push_dialog_action_text(
            commands,
            FrameRect {
                x: action_right,
                y: action_y,
                width: cancel_width,
                height: DIALOG_ACTION_LINE_HEIGHT,
            },
            clip,
            order + 4,
            cancel,
            style::cancel_action_color(unavailable),
            opacity,
        );
        return;
    }

    let Some(action) = action_label(node, 0) else {
        return;
    };
    let width = action_width(&action);
    push_dialog_action_text(
        commands,
        FrameRect {
            x: action_right - width,
            y: action_y,
            width,
            height: DIALOG_ACTION_LINE_HEIGHT,
        },
        clip,
        order + 4,
        action,
        style::dialog_action_color(unavailable),
        opacity,
    );
}
