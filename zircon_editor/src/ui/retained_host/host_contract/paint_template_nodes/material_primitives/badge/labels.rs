use super::super::super::super::data::TemplatePaneNodeData;
use super::super::first_non_empty;

// 根标签读取 text/options_text，计数层读取 value_text/validation_message；候选链不可合并。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_label(
    node: &TemplatePaneNodeData,
) -> String {
    first_non_empty(&[node.text.as_str(), node.options_text.as_str()])
        .trim()
        .to_string()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_display_text(
    node: &TemplatePaneNodeData,
) -> &str {
    first_non_empty(&[node.value_text.as_str(), node.validation_message.as_str()]).trim()
}

#[cfg(test)]
#[path = "labels/tests/capacity_tests.rs"]
mod capacity_tests;
