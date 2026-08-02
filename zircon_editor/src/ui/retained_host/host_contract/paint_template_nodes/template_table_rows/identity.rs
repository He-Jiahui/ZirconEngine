use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    TemplateComponentFamily, is_any_component_family, uses_workbench_visual_language,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_table_row(
    node: &TemplatePaneNodeData,
) -> bool {
    is_any_component_family(
        node,
        &[
            TemplateComponentFamily::Table,
            TemplateComponentFamily::TableRow,
        ],
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_table_row(
    node: &TemplatePaneNodeData,
) -> bool {
    uses_workbench_visual_language(node)
        && is_any_component_family(
            node,
            &[
                TemplateComponentFamily::Table,
                TemplateComponentFamily::TableRow,
            ],
        )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_table_header(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    control_id == "WorkbenchTableHeader" || control_id.ends_with("TableHeader")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_table_tail(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str() == "WorkbenchTableTail"
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_table_selected(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str() == "WorkbenchTableSelected"
}
