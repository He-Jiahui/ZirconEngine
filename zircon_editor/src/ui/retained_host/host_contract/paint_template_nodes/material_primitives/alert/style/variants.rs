use super::super::super::{component_variant_contains, first_non_empty};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

// 调色板和图标共用 severity 选择；显式 variant 优先于校验状态和文字语气。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_color_token(
    node: &TemplatePaneNodeData,
) -> &str {
    for (token, color_token) in [
        ("success", "colorSuccess"),
        ("info", "colorInfo"),
        ("warning", "colorWarning"),
        ("error", "colorError"),
        ("danger", "colorDanger"),
    ] {
        if component_variant_contains(node, token) || component_variant_contains(node, color_token)
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

#[cfg(test)]
#[path = "tests/variants.rs"]
mod tests;
