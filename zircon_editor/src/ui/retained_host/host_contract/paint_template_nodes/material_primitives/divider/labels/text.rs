use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::first_non_empty;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_label(
    node: &TemplatePaneNodeData,
) -> &str {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.options_text.as_str(),
    ])
}

#[cfg(test)]
#[path = "text/capacity_tests.rs"]
mod capacity_tests;
