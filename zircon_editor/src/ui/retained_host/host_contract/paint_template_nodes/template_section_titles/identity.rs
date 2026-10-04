//! 固定工作台 control_id 优先于自由变体扫描；配置驱动标题仍可用 section-title 语义声明。

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
#[path = "identity/tests/fast_control_id_tests.rs"]
mod fast_control_id_tests;
