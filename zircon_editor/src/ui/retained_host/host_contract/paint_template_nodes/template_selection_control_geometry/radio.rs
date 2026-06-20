use super::super::super::data::TemplatePaneNodeData;
use super::metrics::RADIO_DOT_SIZE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn radio_dot_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        RADIO_DOT_SIZE
    }
}
