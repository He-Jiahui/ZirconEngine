use super::super::resolved_state_for_node;
use super::colors::{list_row_adornment_color, list_row_text_color};
use super::model::WorkbenchListRowStyle;
use super::surface::{list_row_background, list_row_border, list_row_border_width};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_list_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchListRowStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::ListRow);
    let row_marked = node.checked || node.selected;
    let adornment_marked = node.checked;
    WorkbenchListRowStyle {
        background: list_row_background(node, state, row_marked),
        border: list_row_border(state),
        border_width: list_row_border_width(state),
        text: list_row_text_color(node, state, row_marked),
        adornment: list_row_adornment_color(node, state, adornment_marked),
        state,
    }
}
