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
