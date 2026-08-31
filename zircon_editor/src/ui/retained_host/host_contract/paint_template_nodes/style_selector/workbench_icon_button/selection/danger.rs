use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_danger_icon(
    node: &TemplatePaneNodeData,
) -> bool {
    let identity_values = [
        node.control_id.as_str(),
        node.icon_name.as_str(),
        node.validation_level.as_str(),
    ];
    ["delete", "trash", "danger", "error"].iter().any(|needle| {
        identity_values
            .iter()
            .any(|value| contains_ignore_ascii_case(value, needle))
    })
}

fn contains_ignore_ascii_case(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
