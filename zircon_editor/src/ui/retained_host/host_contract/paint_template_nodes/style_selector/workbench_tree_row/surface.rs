//! 持久选择背景由所属树的真实焦点决定；焦点可见轮廓独立于选择与主交互态。
//! 宽度随轮廓是否存在同步给绘制端。

use super::palette::{workbench_tree_row_palette, WorkbenchTreeRowPalette};
use super::state::{is_hot, is_unavailable_tree_row_state, uses_focus_outline};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    tree_row_background_from_palette(node, state, marked, workbench_tree_row_palette())
}

fn tree_row_background_from_palette(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchTreeRowPalette,
) -> Option<[u8; 4]> {
    if is_unavailable_tree_row_state(state) {
        None
    } else if marked {
        Some(if node.tree_owner_focused == Some(false) {
            palette.inactive_marked_surface
        } else {
            palette.marked_surface
        })
    } else if state == UiPainterResolvedState::Pressed {
        Some(palette.pressed_surface)
    } else if is_hot(state) {
        Some(palette.hot_surface)
    } else {
        None
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border(
    state: UiPainterResolvedState,
    focus_visible: bool,
) -> Option<[u8; 4]> {
    tree_row_border_from_palette(state, focus_visible, workbench_tree_row_palette())
}

fn tree_row_border_from_palette(
    state: UiPainterResolvedState,
    focus_visible: bool,
    palette: WorkbenchTreeRowPalette,
) -> Option<[u8; 4]> {
    if is_unavailable_tree_row_state(state) {
        None
    } else if focus_visible || uses_focus_outline(state) {
        Some(palette.focus_border)
    } else {
        None
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_border_width(
    state: UiPainterResolvedState,
    focus_visible: bool,
) -> f32 {
    if tree_row_border(state, focus_visible).is_some() {
        1.0
    } else {
        0.0
    }
}
