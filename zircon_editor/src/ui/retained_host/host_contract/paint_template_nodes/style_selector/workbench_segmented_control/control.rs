use super::super::super::template_style_color::resolved_style_color;
use super::model::WorkbenchSegmentedControlKind;
use super::palette::WORKBENCH_SEGMENT_IDLE_BACKGROUND;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_background(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSegmentedControlKind,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            Some(PALETTE.surface_disabled)
        }
        UiPainterResolvedState::Pressed => Some(PALETTE.surface_pressed),
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => Some(PALETTE.surface_hover),
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => match kind {
            WorkbenchSegmentedControlKind::SegmentedControl => {
                Some(WORKBENCH_SEGMENT_IDLE_BACKGROUND)
            }
            WorkbenchSegmentedControlKind::Tab => {
                resolved_style_color(node.button_style.element.background_color.as_ref())
            }
        },
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_border(
    kind: WorkbenchSegmentedControlKind,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    match kind {
        WorkbenchSegmentedControlKind::Tab => None,
        WorkbenchSegmentedControlKind::SegmentedControl => Some(match state {
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                PALETTE.border_disabled
            }
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => PALETTE.accent,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => PALETTE.border,
        }),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_border_width(
    kind: WorkbenchSegmentedControlKind,
) -> f32 {
    match kind {
        WorkbenchSegmentedControlKind::SegmentedControl => 1.0,
        WorkbenchSegmentedControlKind::Tab => 0.0,
    }
}
