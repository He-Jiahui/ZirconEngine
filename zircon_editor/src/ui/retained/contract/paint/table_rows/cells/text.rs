use super::super::super::super::data::TemplatePaneNodeData;
use std::borrow::Cow;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn table_cells(
    node: &TemplatePaneNodeData,
) -> Cow<'_, [String]> {
    node.table_cells()
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use
    super::super::super::super::data::split_archived_table_text;
