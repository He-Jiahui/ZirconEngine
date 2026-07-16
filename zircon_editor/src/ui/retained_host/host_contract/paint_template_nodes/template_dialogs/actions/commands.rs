use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::identity::DialogKind;
use super::super::{layout, metrics::dialog_metrics, style};
use super::labels::{action_label, action_width};
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
) -> Option<f32> {
    if matches!(kind, DialogKind::ConfirmDialog) {
        let confirm = action_label(node, 1).unwrap_or_else(|| "Confirm".to_string());
        let confirm_width = action_width(&confirm);
        let confirm_enabled = style::confirm_enabled(node) && !unavailable;
        let cancel = action_label(node, 0).unwrap_or_else(|| "Cancel".to_string());
        let frames = confirm_action_frames(rect, action_width(&cancel), confirm_width);
        push_dialog_action_text(
            commands,
            frames.confirm.clone(),
            clip,
            order + 5,
            confirm,
            style::confirm_action_color(node, unavailable, confirm_enabled),
            opacity,
        );
        push_dialog_action_text(
            commands,
            frames.cancel.clone(),
            clip,
            order + 4,
            cancel,
            style::cancel_action_color(unavailable),
            opacity,
        );
        return Some(frames.cancel.y);
    }

    if matches!(kind, DialogKind::AlertDialog) {
        push_legacy_confirm_actions(commands, node, rect, clip, order, unavailable, opacity);
        return None;
    }

    let Some(action) = action_label(node, 0) else {
        return None;
    };
    let frame = single_action_frame(rect, action_width(&action));
    push_dialog_action_text(
        commands,
        frame.clone(),
        clip,
        order + 4,
        action,
        style::dialog_action_color(unavailable),
        opacity,
    );
    Some(frame.y)
}

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
    let preferred_bottom_y =
        rect.y + rect.height - metrics.action_bottom - metrics.action_line_height;
    let bottom_y = preferred_bottom_y.max(action_rail_floor + metrics.action_line_height);
    let action_right = layout::action_right(rect);

    if cancel_width + metrics.action_gap + confirm_width <= available_width {
        let confirm = action_frame(
            action_right,
            bottom_y,
            confirm_width,
            metrics.action_line_height,
        );
        return ConfirmActionFrames {
            cancel: action_frame(
                confirm.x - metrics.action_gap,
                bottom_y,
                cancel_width,
                metrics.action_line_height,
            ),
            confirm,
            stacked: false,
        };
    }

    let available_stack_gap = (bottom_y - metrics.action_line_height - action_rail_floor).max(0.0);
    let stack_gap = metrics.action_stack_gap.min(available_stack_gap);
    let stacked_cancel_y =
        (bottom_y - stack_gap - metrics.action_line_height).max(action_rail_floor);
    ConfirmActionFrames {
        cancel: action_frame(
            action_right,
            stacked_cancel_y,
            cancel_width,
            metrics.action_line_height,
        ),
        confirm: action_frame(
            action_right,
            bottom_y,
            confirm_width,
            metrics.action_line_height,
        ),
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
    let metrics = dialog_metrics();
    let action_y = rect.y + rect.height - metrics.legacy_action_bottom - metrics.action_line_height;
    let mut action_right = layout::action_right(rect);
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
            height: metrics.action_line_height,
        },
        clip,
        order + 5,
        confirm,
        style::confirm_action_color(node, unavailable, confirm_enabled),
        opacity,
    );
    action_right -= metrics.action_gap;

    let cancel = action_label(node, 0).unwrap_or_else(|| "Cancel".to_string());
    let cancel_width = action_width(&cancel);
    action_right -= cancel_width;
    push_dialog_action_text(
        commands,
        FrameRect {
            x: action_right,
            y: action_y,
            width: cancel_width,
            height: metrics.action_line_height,
        },
        clip,
        order + 4,
        cancel,
        style::cancel_action_color(unavailable),
        opacity,
    );
}

fn single_action_frame(rect: &FrameRect, action_width: f32) -> FrameRect {
    let metrics = dialog_metrics();
    action_frame(
        layout::action_right(rect),
        rect.y + rect.height - metrics.action_bottom - metrics.action_line_height,
        action_width.min(layout::action_available_width(rect)),
        metrics.action_line_height,
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
mod tests {
    use super::*;

    #[test]
    fn narrow_confirm_actions_stack_without_overlapping_each_other() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 154.0,
            height: 120.0,
        };
        let frames = confirm_action_frames(&rect, 64.0, 64.0);

        assert!(frames.cancel.y + frames.cancel.height <= frames.confirm.y);
        assert_eq!(frames.cancel.x, frames.confirm.x);
        assert!(frames.stacked);
        assert!(frames.cancel.width <= layout::action_available_width(&rect));
        assert!(frames.confirm.width <= layout::action_available_width(&rect));
    }

    #[test]
    fn short_narrow_confirm_actions_compact_the_stack_gap_without_clipping_labels() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 154.0,
            height: 88.0,
        };
        let frames = confirm_action_frames(&rect, 64.0, 64.0);

        assert!(frames.stacked);
        assert_eq!(frames.cancel.x, frames.confirm.x);
        assert_eq!(frames.cancel.width, 64.0);
        assert_eq!(frames.confirm.width, 64.0);
        assert!(frames.cancel.y + frames.cancel.height <= frames.confirm.y);
        assert!(frames.cancel.y >= layout::action_rail_floor(&rect));
    }
}
