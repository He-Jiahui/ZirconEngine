use super::super::resolved_state_for_node;
use super::colors::apply_declared_toast_colors;
use super::model::WorkbenchToastStyle;
use super::state::toast_state_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_toast_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchToastStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Toast);
    let mut style = toast_state_style(state);

    apply_declared_toast_colors(node, &mut style);

    style
}
