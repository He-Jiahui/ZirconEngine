//! 专用 workbench tooltip 与普通 tooltip 区分；固定根 ID 和明确 surface_variant 都可触发认领。

use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_tooltip(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str() == "WorkbenchTooltipRoot"
        || node.surface_variant.as_str() == "workbench-tooltip"
        || (node.control_id.as_str().starts_with("Workbench")
            && (node.role.as_str() == "Tooltip" || node.component_role.as_str() == "tooltip"))
}
