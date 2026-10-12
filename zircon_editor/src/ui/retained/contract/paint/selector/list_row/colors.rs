//! 列表行正文按已标记状态在主文字与次文字之间选择；勾选标记的装饰色另由 checked 决定，节点声明色只覆盖各自通道。
//! 禁用或加载优先使用禁用文字角色；声明色透明度为零时退回当前主题角色。

use super::palette::{workbench_list_row_palette, WorkbenchListRowPalette};
use super::state::is_unavailable_list_row_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_text_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    list_row_text_color_from_palette(node, state, marked, workbench_list_row_palette())
}

fn list_row_text_color_from_palette(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchListRowPalette,
) -> [u8; 4] {
    if is_unavailable_list_row_state(state) {
        palette.text_disabled
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else if marked {
        palette.text
    } else {
        palette.text_muted
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_adornment_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    list_row_adornment_color_from_palette(node, state, marked, workbench_list_row_palette())
}

fn list_row_adornment_color_from_palette(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    palette: WorkbenchListRowPalette,
) -> [u8; 4] {
    if is_unavailable_list_row_state(state) {
        palette.text_disabled
    } else if let Some(color) = declared_color(node.icon_color) {
        color
    } else if marked {
        palette.marked_adornment
    } else {
        palette.text_muted
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
