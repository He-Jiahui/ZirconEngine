//! 选择控件的表面通道；持久勾选与动态交互分开传入，Focused 开关仍保留同时悬停的表面反馈。
//! 禁用和加载统一覆盖声明表面；已勾选配方不接受未勾选的声明表面覆盖。

use super::super::colors::declared_style_background;
use super::super::model::WorkbenchSelectionControlKind;
use super::super::palette::WorkbenchSelectionControlPalette;
use super::super::state::{is_hot, is_unavailable_selection_state, selection_node_is_hot};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(super) fn control_surface(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
    state: UiPainterResolvedState,
    checked: bool,
    palette: WorkbenchSelectionControlPalette,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        return palette.surface_disabled;
    }
    match kind {
        WorkbenchSelectionControlKind::Checkbox => {
            if checked {
                palette.checkbox_checked_fill
            } else {
                declared_style_background(node).unwrap_or(palette.mark_idle_fill)
            }
        }
        WorkbenchSelectionControlKind::Radio => {
            if checked {
                palette.radio_checked_fill
            } else {
                declared_style_background(node).unwrap_or(palette.mark_idle_fill)
            }
        }
        WorkbenchSelectionControlKind::Toggle => {
            if checked {
                palette.toggle_checked_surface
            } else if state == UiPainterResolvedState::Pressed {
                palette.toggle_pressed_surface
            } else if state == UiPainterResolvedState::Focused && selection_node_is_hot(node) {
                palette.toggle_hover_surface
            } else if is_hot(state) {
                palette.toggle_hover_surface
            } else {
                declared_style_background(node).unwrap_or(palette.toggle_track)
            }
        }
    }
}
