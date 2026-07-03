use super::super::resolved_state_for_node;
use super::colors::{
    declared_value_color, table_row_action_color, table_row_background, table_row_border,
    table_row_border_width,
};
use super::identity::{is_table_header, is_table_tail};
use super::model::WorkbenchTableRowStyle;
use super::palette::workbench_table_row_palette;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_table_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTableRowStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::TableRow);
    let marked = node.selected || node.checked;
    let header = is_table_header(node);
    let tail = is_table_tail(node);
    let palette = workbench_table_row_palette();

    WorkbenchTableRowStyle {
        background: table_row_background(node, state, marked, header, tail),
        border: table_row_border(state, marked),
        border_width: table_row_border_width(state, marked),
        separator: palette.separator,
        action: table_row_action_color(state),
        state,
        text: palette.text,
        muted_text: palette.text_muted,
        tail_value_text: declared_value_color(node).unwrap_or(palette.tail_value_text),
        header,
        tail,
    }
}
