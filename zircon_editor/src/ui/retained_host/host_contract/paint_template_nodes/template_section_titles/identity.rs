use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_section_title(
    node: &TemplatePaneNodeData,
) -> bool {
    if matches!(
        node.control_id.as_str(),
        "WorkbenchSectionTitleRoot" | "WorkbenchTransformLabel" | "WorkbenchMeshLabel"
    ) {
        return true;
    }
    node.component_variant
        .split_ascii_whitespace()
        .any(|variant| variant == "section-title")
}

#[cfg(test)]
#[path = "identity/fast_control_id_tests.rs"]
mod fast_control_id_tests;
