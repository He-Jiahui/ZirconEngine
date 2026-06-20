use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_label(
    node: &TemplatePaneNodeData,
) -> Option<String> {
    let label = node.label_text.trim();
    (!label.is_empty()).then(|| label.to_owned())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_label(
    node: &TemplatePaneNodeData,
    percent: f32,
) -> String {
    let value = node.value_text.trim();
    if value.is_empty() {
        format!("{:.2}", percent.clamp(0.0, 1.0))
    } else {
        value.to_owned()
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_range_min_label(
    percent: f32,
) -> String {
    format!("{:.2}", percent.clamp(0.0, 1.0))
}
