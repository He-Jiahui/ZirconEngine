//! 表格行先处理不可用、按压、持久选择和热态，再回到表头/表尾身份及普通声明色。
//! 表头背景优先于普通声明色，表尾在没有声明背景时使用宿主表格基色。

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

use super::super::palette::workbench_table_row_palette;
use super::super::state::{is_hot, is_unavailable_table_row_state};
use super::declared::declared_background_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_row_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    header: bool,
    tail: bool,
) -> [u8; 4] {
    let palette = workbench_table_row_palette();
    if is_unavailable_table_row_state(state) {
        palette.surface_disabled
    } else if state == UiPainterResolvedState::Pressed {
        palette.surface_pressed
    } else if marked {
        palette.selected_bg
    } else if is_hot(state) {
        palette.hover_bg
    } else if header {
        palette.header_bg
    } else {
        declared_background_color(node).unwrap_or_else(|| {
            if tail {
                palette.tail_bg
            } else {
                palette.row_bg
            }
        })
    }
}
