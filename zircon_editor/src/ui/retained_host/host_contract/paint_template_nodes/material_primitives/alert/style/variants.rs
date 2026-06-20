use super::super::super::{component_variant_contains, first_non_empty};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_color_token(
    node: &TemplatePaneNodeData,
) -> &str {
    for token in ["success", "info", "warning", "error", "danger"] {
        if component_variant_contains(node, token)
            || component_variant_contains(node, &format!("color{}", pascal_case(token)))
        {
            return token;
        }
    }
    match first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()]) {
        "success" => "success",
        "info" => "info",
        "warning" => "warning",
        "error" | "danger" => "error",
        _ => "success",
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_is_filled(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "filled")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_is_outlined(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "outlined")
}

fn pascal_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + characters.as_str()
}
