//! 属性行复用工作台密度，组件属性标签偏好宽度只是该密度下的投影，不形成单独配置源。

use super::super::super::template_row_metrics::{workbench_row_metrics, WorkbenchRowMetrics};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_row_metrics(
) -> WorkbenchRowMetrics {
    workbench_row_metrics()
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn component_property_label_width(
) -> f32 {
    property_row_metrics().component_property_label_width
}
