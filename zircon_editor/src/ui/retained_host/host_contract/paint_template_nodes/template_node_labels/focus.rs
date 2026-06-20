use super::super::super::data::{HostTextInputFocusData, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn focused_text_value<'a>(
    node: &TemplatePaneNodeData,
    text_input_focus: Option<&'a HostTextInputFocusData>,
) -> Option<&'a str> {
    let focus = text_input_focus?;
    (focus.is_active() && focus.control_id.as_str() == node.control_id.as_str())
        .then_some(focus.value_text.as_str())
}
