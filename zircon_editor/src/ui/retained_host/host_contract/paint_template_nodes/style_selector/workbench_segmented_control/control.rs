use super::super::super::template_style_color::resolved_style_color;
use super::metrics::workbench_segmented_selector_metrics;
use super::model::WorkbenchSegmentedControlKind;
use super::palette::workbench_segmented_control_palette;
use super::state::segmented_node_is_hot;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_background(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSegmentedControlKind,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    let palette = workbench_segmented_control_palette();
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            Some(palette.disabled_background)
        }
        UiPainterResolvedState::Pressed => Some(palette.pressed_background),
        UiPainterResolvedState::Focused => {
            if segmented_node_is_hot(node) {
                Some(palette.hot_background)
            } else {
                normal_control_background(node, kind)
            }
        }
        UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => Some(palette.hot_background),
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => normal_control_background(node, kind),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_border(
    kind: WorkbenchSegmentedControlKind,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    let palette = workbench_segmented_control_palette();
    match kind {
        WorkbenchSegmentedControlKind::Tab => None,
        WorkbenchSegmentedControlKind::SegmentedControl => Some(match state {
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                palette.disabled_border
            }
            UiPainterResolvedState::Focused => palette.focus_border,
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => palette.active_border,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => palette.border,
        }),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn control_border_width(
    kind: WorkbenchSegmentedControlKind,
) -> f32 {
    match kind {
        WorkbenchSegmentedControlKind::SegmentedControl => {
            workbench_segmented_selector_metrics().border_width
        }
        WorkbenchSegmentedControlKind::Tab => 0.0,
    }
}

fn normal_control_background(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSegmentedControlKind,
) -> Option<[u8; 4]> {
    let palette = workbench_segmented_control_palette();
    match kind {
        WorkbenchSegmentedControlKind::SegmentedControl => Some(palette.idle_background),
        WorkbenchSegmentedControlKind::Tab => {
            resolved_style_color(node.button_style.element.background_color.as_ref())
        }
    }
}
