use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_danger_icon(
    node: &TemplatePaneNodeData,
) -> bool {
    let key = format!(
        "{} {} {}",
        node.control_id.as_str(),
        node.icon_name.as_str(),
        node.validation_level.as_str()
    )
    .to_ascii_lowercase();
    key.contains("delete")
        || key.contains("trash")
        || key.contains("danger")
        || key.contains("error")
}
