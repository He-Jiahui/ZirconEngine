use super::super::resolved_state_for_node;
use super::colors::apply_declared_tooltip_colors;
use super::model::WorkbenchTooltipStyle;
use super::state::tooltip_state_style;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_tooltip_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTooltipStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Tooltip);
    let mut style = tooltip_state_style(state);

    apply_declared_tooltip_colors(node, &mut style);

    style
}
