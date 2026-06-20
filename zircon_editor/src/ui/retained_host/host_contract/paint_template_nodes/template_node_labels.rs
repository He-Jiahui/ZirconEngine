use super::super::data::{HostTextInputFocusData, TemplatePaneNodeData};

mod focus;
mod property;
mod values;

use focus::focused_text_value;
use property::property_row_label;
use values::fallback_node_label;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_node_label(
    node: &TemplatePaneNodeData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> String {
    if let Some(focus) = focused_text_value(node, text_input_focus) {
        return focus.to_string();
    }
    if let Some(property_label) = property_row_label(node) {
        return property_label;
    }
    fallback_node_label(node)
}

#[cfg(test)]
#[path = "template_node_labels_tests.rs"]
mod tests;
