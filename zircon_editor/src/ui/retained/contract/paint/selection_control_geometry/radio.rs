//! 单选点大小允许正的实例 value_number 覆盖默认；最终可见点由共享居中框限制在标记内。

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
