use super::super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_paper_root_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "paper" | "Paper" | "mui-paper" | "MuiPaper"
    ) || matches!(node.role.as_str(), "Paper" | "MuiPaper")
}
