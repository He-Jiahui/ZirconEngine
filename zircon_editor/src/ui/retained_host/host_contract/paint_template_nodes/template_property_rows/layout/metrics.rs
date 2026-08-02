use super::super::super::template_row_metrics::{WorkbenchRowMetrics, workbench_row_metrics};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_row_metrics()
-> WorkbenchRowMetrics {
    workbench_row_metrics()
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn component_property_label_width()
-> f32 {
    property_row_metrics().component_property_label_width
}
