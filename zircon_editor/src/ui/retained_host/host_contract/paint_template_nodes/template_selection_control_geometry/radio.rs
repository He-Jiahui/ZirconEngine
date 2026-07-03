use super::super::super::data::TemplatePaneNodeData;
use super::metrics::workbench_selection_control_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn radio_dot_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        workbench_selection_control_metrics().radio_dot_size
    }
}
