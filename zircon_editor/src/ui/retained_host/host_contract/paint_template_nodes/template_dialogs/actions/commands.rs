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
#[path = "commands/dialog_kind_dispatch_tests.rs"]
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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

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

    #[test]
    fn confirm_actions_paint_standard_secondary_and_primary_button_surfaces() {
        let node = TemplatePaneNodeData::default();
        let rect = FrameRect {
            x: 8.0,
            y: 12.0,
            width: 240.0,
            height: 132.0,
        };
        let mut commands = Vec::new();

        let action_top = push_dialog_actions(
            &mut commands,
            &node,
            &rect,
            &rect,
            10,
            DialogKind::ConfirmDialog,
            false,
            1.0,
        )
        .expect("confirm dialogs should reserve their action rail");

        let surfaces = commands
            .iter()
            .filter(|command| matches!(command.kind, HostPaintCommandKind::Quad))
            .collect::<Vec<_>>();

        assert_eq!(surfaces.len(), 2);
        assert_eq!(surfaces[0].background_color, Some(PALETTE.surface));
        assert_eq!(surfaces[0].border_color, Some(PALETTE.border));
        assert_eq!(surfaces[1].background_color, Some(PALETTE.accent));
        assert_eq!(surfaces[1].border_color, Some(PALETTE.accent));
        assert_eq!(surfaces[0].frame.height, METRICS.row_height);
        assert_eq!(surfaces[1].frame.height, METRICS.row_height);
        assert_eq!(surfaces[0].frame.y, action_top);
        assert!(surfaces.iter().all(|surface| surface.frame.x >= rect.x
            && surface.frame.x + surface.frame.width <= rect.x + rect.width));
    }

    #[test]
    fn legacy_alert_actions_share_narrow_width_without_losing_the_cancel_surface() {
        let rect = FrameRect {
            x: 8.0,
            y: 12.0,
            width: 152.0,
            height: 144.0,
        };
        let mut commands = Vec::new();

        push_dialog_actions(
            &mut commands,
            &TemplatePaneNodeData::default(),
            &rect,
            &rect,
            10,
            DialogKind::AlertDialog,
            false,
            1.0,
        );

        let surfaces = commands
            .iter()
            .filter(|command| matches!(command.kind, HostPaintCommandKind::Quad))
            .collect::<Vec<_>>();

        assert_eq!(surfaces.len(), 2);
        assert!(surfaces.iter().all(|surface| surface.frame.width > 0.0));
        assert!(surfaces.iter().all(|surface| surface.frame.x >= rect.x
            && surface.frame.x + surface.frame.width <= rect.x + rect.width));
        assert_eq!(surfaces[0].frame.y, surfaces[1].frame.y);
    }
}
