use super::super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_material_progress_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
    ) || matches!(
        node.role.as_str(),
        "Progress" | "ProgressBar" | "LinearProgress" | "CircularProgress" | "Spinner"
    )
}
