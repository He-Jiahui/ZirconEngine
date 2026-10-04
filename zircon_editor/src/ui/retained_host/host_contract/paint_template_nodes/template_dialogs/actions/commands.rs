//! 动作带根据dialog种类采用不同布局合同：ConfirmDialog可堆叠，AlertDialog维持单行，普通dialog消费首个动作。
//! 返回值供正文避让；此层只消费动作label和可用状态，不以视觉命令替代action_id的执行路由。

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
#[cfg(test)]
use super::super::super::render_commands::HostPaintCommandKind;
use super::super::identity::DialogKind;
use super::super::{layout, metrics::dialog_metrics, style};
use super::labels::{action_label, action_text_frame, action_width};
use super::surface::push_dialog_action_surface;
use super::text::push_dialog_action_text;

const LEGACY_ACTION_GAP_MAX_FRACTION: f32 = 0.2;

/// 确认/警告类按 actions[0]取消、actions[1]确认的投影约定显示；普通对话框只显示首个动作。
/// 上层使用返回的最早动作位置为正文留空间；没有返回值不能自动判断是否生成了AlertDialog动作。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_dialog_actions(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: DialogKind,
    unavailable: bool,
    opacity: f32,
) -> Option<f32> {
    match kind {
        DialogKind::ConfirmDialog => {
            let confirm = action_label(node, 1).unwrap_or_else(|| "Confirm".to_string());
            let confirm_width = action_width(&confirm);
            let confirm_enabled = style::confirm_enabled(node) && !unavailable;
            let cancel = action_label(node, 0).unwrap_or_else(|| "Cancel".to_string());
            let frames = confirm_action_frames(rect, action_width(&cancel), confirm_width);
            let cancel_paint = style::cancel_action_paint(unavailable);
            let confirm_paint = style::confirm_action_paint(node, unavailable, confirm_enabled);
            push_dialog_action_surface(
                commands,
                rect,
                frames.cancel.clone(),
                clip,
                order + 4,
                cancel_paint,
                opacity,
            );
            push_dialog_action_text(
                commands,
                rect,
                action_text_frame(&frames.cancel, &cancel),
                clip,
                order + 5,
                cancel,
                cancel_paint.text,
                opacity,
            );
            push_dialog_action_surface(
                commands,
                rect,
                frames.confirm.clone(),
                clip,
                order + 6,
                confirm_paint,
                opacity,
            );
            push_dialog_action_text(
                commands,
                rect,
                action_text_frame(&frames.confirm, &confirm),
                clip,
                order + 7,
                confirm,
                confirm_paint.text,
                opacity,
            );
            return Some(frames.cancel.y);
        }
        DialogKind::AlertDialog => {
            push_legacy_confirm_actions(commands, node, rect, clip, order, unavailable, opacity);
            None
        }
        _ => {
            let Some(action) = action_label(node, 0) else {
                return None;
            };
            let frame = single_action_frame(rect, action_width(&action));
            let paint = style::dialog_action_paint(unavailable);
            push_dialog_action_surface(
                commands,
                rect,
                frame.clone(),
                clip,
                order + 4,
                paint,
                opacity,
            );
            push_dialog_action_text(
                commands,
                rect,
                action_text_frame(&frame, &action),
                clip,
                order + 5,
                action,
                paint.text,
                opacity,
            );
            Some(frame.y)
        }
    }
}

#[cfg(test)]
#[path = "commands/tests/dialog_kind_dispatch_tests.rs"]
mod dialog_kind_dispatch_tests;

#[derive(Clone, Debug)]
struct ConfirmActionFrames {
    cancel: FrameRect,
    confirm: FrameRect,
    stacked: bool,
}

fn confirm_action_frames(
    rect: &FrameRect,
    cancel_width: f32,
    confirm_width: f32,
) -> ConfirmActionFrames {
    let metrics = dialog_metrics();
    let available_width = layout::action_available_width(rect);
    let cancel_width = cancel_width.min(available_width);
    let confirm_width = confirm_width.min(available_width);
    let action_rail_floor = layout::action_rail_floor(rect);
    let preferred_bottom_y = rect.y + rect.height - metrics.action_bottom - metrics.action_height;
    let bottom_y = preferred_bottom_y.max(action_rail_floor + metrics.action_height);
    let action_right = layout::action_right(rect);

    if cancel_width + metrics.action_gap + confirm_width <= available_width {
        let confirm = action_frame(action_right, bottom_y, confirm_width, metrics.action_height);
        return ConfirmActionFrames {
            cancel: action_frame(
                confirm.x - metrics.action_gap,
                bottom_y,
                cancel_width,
                metrics.action_height,
            ),
            confirm,
            stacked: false,
        };
    }

    let available_stack_gap = (bottom_y - metrics.action_height - action_rail_floor).max(0.0);
    let stack_gap = metrics.action_stack_gap.min(available_stack_gap);
    let stacked_cancel_y = (bottom_y - stack_gap - metrics.action_height).max(action_rail_floor);
    ConfirmActionFrames {
        cancel: action_frame(
            action_right,
            stacked_cancel_y,
            cancel_width,
            metrics.action_height,
        ),
        confirm: action_frame(action_right, bottom_y, confirm_width, metrics.action_height),
        stacked: true,
    }
}

fn push_legacy_confirm_actions(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    unavailable: bool,
    opacity: f32,
) {
    let confirm = action_label(node, 1).unwrap_or_else(|| "Confirm".to_string());
    let confirm_enabled = style::confirm_enabled(node) && !unavailable;
    let cancel = action_label(node, 0).unwrap_or_else(|| "Cancel".to_string());
    let frames = legacy_confirm_action_frames(rect, action_width(&cancel), action_width(&confirm));
    let confirm_paint = style::confirm_action_paint(node, unavailable, confirm_enabled);
    push_dialog_action_surface(
        commands,
        rect,
        frames.confirm.clone(),
        clip,
        order + 6,
        confirm_paint,
        opacity,
    );
    push_dialog_action_text(
        commands,
        rect,
        action_text_frame(&frames.confirm, &confirm),
        clip,
        order + 7,
        confirm,
        confirm_paint.text,
        opacity,
    );

    let cancel_paint = style::cancel_action_paint(unavailable);
    push_dialog_action_surface(
        commands,
        rect,
        frames.cancel.clone(),
        clip,
        order + 4,
        cancel_paint,
        opacity,
    );
    push_dialog_action_text(
        commands,
        rect,
        action_text_frame(&frames.cancel, &cancel),
        clip,
        order + 5,
        cancel,
        cancel_paint.text,
        opacity,
    );
}

fn legacy_confirm_action_frames(
    rect: &FrameRect,
    cancel_width: f32,
    confirm_width: f32,
) -> ConfirmActionFrames {
    let metrics = dialog_metrics();
    let available_width = layout::action_available_width(rect);
    // Alert dialogs retain their one-row layout, so narrow surfaces compress both buttons
    // together instead of letting the left action fall outside the dialog clip.
    let action_gap = metrics
        .action_gap
        .min(available_width * LEGACY_ACTION_GAP_MAX_FRACTION);
    let button_width = (available_width - action_gap).max(0.0);
    let (cancel_width, confirm_width) =
        proportional_action_widths(cancel_width, confirm_width, button_width);
    let action_y = rect.y + rect.height - metrics.legacy_action_bottom - metrics.action_height;
    let confirm = action_frame(
        layout::action_right(rect),
        action_y,
        confirm_width,
        metrics.action_height,
    );
    ConfirmActionFrames {
        cancel: action_frame(
            confirm.x - action_gap,
            action_y,
            cancel_width,
            metrics.action_height,
        ),
        confirm,
        stacked: false,
    }
}

fn proportional_action_widths(
    cancel_width: f32,
    confirm_width: f32,
    available_width: f32,
) -> (f32, f32) {
    let preferred_width = cancel_width + confirm_width;
    if preferred_width <= available_width {
        return (cancel_width, confirm_width);
    }
    if preferred_width <= 0.0 {
        return (available_width / 2.0, available_width / 2.0);
    }

    let cancel_width = available_width * cancel_width / preferred_width;
    (cancel_width, (available_width - cancel_width).max(0.0))
}

fn single_action_frame(rect: &FrameRect, action_width: f32) -> FrameRect {
    let metrics = dialog_metrics();
    action_frame(
        layout::action_right(rect),
        rect.y + rect.height - metrics.action_bottom - metrics.action_height,
        action_width.min(layout::action_available_width(rect)),
        metrics.action_height,
    )
}

fn action_frame(action_right: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: action_right - width,
        y,
        width,
        height,
    }
}

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
