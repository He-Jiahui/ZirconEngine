use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;
use super::style_selector::select_workbench_list_row_style;

const LIST_ROW_RIGHT_INSET: f32 = 12.0;
const LIST_ROW_ADORNMENT_SIZE: f32 = 13.0;

pub(super) fn push_list_row_adornment(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let adornment = list_row_adornment_rect(rect);
    match list_row_adornment_kind(node) {
        ListRowAdornmentKind::Check => {
            push_check_mark(commands, &adornment, clip, order, color, opacity);
        }
        ListRowAdornmentKind::Chevron => {
            push_right_chevron(commands, &adornment, clip, order, color, opacity);
        }
        ListRowAdornmentKind::DisabledDiamond => {
            push_disabled_diamond(commands, &adornment, clip, order, opacity);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ListRowAdornmentKind {
    Check,
    Chevron,
    DisabledDiamond,
}

pub(super) fn list_row_adornment_kind(node: &TemplatePaneNodeData) -> ListRowAdornmentKind {
    if is_unavailable_list_row_state(select_workbench_list_row_style(node).state) {
        ListRowAdornmentKind::DisabledDiamond
    } else if node.checked || node.selected {
        ListRowAdornmentKind::Check
    } else {
        ListRowAdornmentKind::Chevron
    }
}

fn is_unavailable_list_row_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn list_row_adornment_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - LIST_ROW_RIGHT_INSET - LIST_ROW_ADORNMENT_SIZE,
        y: rect.y + (rect.height - LIST_ROW_ADORNMENT_SIZE).max(0.0) * 0.5,
        width: LIST_ROW_ADORNMENT_SIZE,
        height: LIST_ROW_ADORNMENT_SIZE,
    }
}

fn push_check_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in [
        FrameRect {
            x: rect.x + 2.0,
            y: rect.y + 7.0,
            width: 3.0,
            height: 2.0,
        },
        FrameRect {
            x: rect.x + 4.0,
            y: rect.y + 9.0,
            width: 3.0,
            height: 2.0,
        },
        FrameRect {
            x: rect.x + 7.0,
            y: rect.y + 4.0,
            width: 3.0,
            height: 7.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_right_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in [
        FrameRect {
            x: rect.x + 5.0,
            y: rect.y + 3.0,
            width: 2.0,
            height: 3.0,
        },
        FrameRect {
            x: rect.x + 7.0,
            y: rect.y + 6.0,
            width: 2.0,
            height: 2.0,
        },
        FrameRect {
            x: rect.x + 5.0,
            y: rect.y + 8.0,
            width: 2.0,
            height: 3.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn push_disabled_diamond(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = PALETTE.text_disabled;
    let center_x = rect.x + rect.width * 0.5;
    let center_y = rect.y + rect.height * 0.5;
    for segment in [
        FrameRect {
            x: center_x - 1.0,
            y: center_y - 5.0,
            width: 2.0,
            height: 2.0,
        },
        FrameRect {
            x: center_x + 3.0,
            y: center_y - 1.0,
            width: 2.0,
            height: 2.0,
        },
        FrameRect {
            x: center_x - 1.0,
            y: center_y + 3.0,
            width: 2.0,
            height: 2.0,
        },
        FrameRect {
            x: center_x - 5.0,
            y: center_y - 1.0,
            width: 2.0,
            height: 2.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}
