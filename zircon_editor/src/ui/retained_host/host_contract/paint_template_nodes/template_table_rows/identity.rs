use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_any_component_family, uses_workbench_visual_language, TemplateComponentFamily,
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
    node.control_id.as_str() == "WorkbenchTableHeader"
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
