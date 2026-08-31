use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::super::style_selector::{is_tab_like_workbench_button, WorkbenchButtonKind};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_button(
    node: &TemplatePaneNodeData,
) -> bool {
    let control_id = node.control_id.as_str();
    (uses_workbench_visual_language(node)
        || uses_editor_button_variant(node)
        || is_tab_like_workbench_button(node))
        && !control_id.starts_with("WorkbenchDrawerTab")
        && !control_id.starts_with("WorkbenchTool")
        && !control_id.starts_with("WorkbenchToolbar")
        && !control_id.starts_with("WorkbenchRail")
        && !control_id.starts_with("WorkbenchStatus")
        && !control_id.starts_with("WorkbenchMini")
        && !control_id.contains("IconButton")
        && is_component_family(node, TemplateComponentFamily::Button)
}

fn uses_editor_button_variant(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.button_variant.as_str(),
        "primary" | "secondary" | "tertiary" | "filled" | "outlined" | "text" | "ghost" | "danger"
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_kind(
    node: &TemplatePaneNodeData,
) -> WorkbenchButtonKind {
    let values = button_identity_values(node);
    if ["danger", "delete", "trash"]
        .iter()
        .any(|needle| button_identity_contains(&values, needle))
    {
        WorkbenchButtonKind::Danger
    } else if ["primary", "filled", "accent"]
        .iter()
        .any(|needle| button_identity_contains(&values, needle))
    {
        WorkbenchButtonKind::Primary
    } else if ["tertiary", "text"]
        .iter()
        .any(|needle| button_identity_contains(&values, needle))
    {
        WorkbenchButtonKind::Tertiary
    } else {
        WorkbenchButtonKind::Secondary
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_add_component_button(
    node: &TemplatePaneNodeData,
) -> bool {
    node.control_id.as_str() == "WorkbenchAddComponent"
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_identity_values(
    node: &TemplatePaneNodeData,
) -> [&str; 6] {
    [
        node.control_id.as_str(),
        node.text.as_str(),
        node.value_text.as_str(),
        node.button_variant.as_str(),
        node.surface_variant.as_str(),
        node.validation_level.as_str(),
    ]
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_identity_contains(
    values: &[&str],
    needle: &str,
) -> bool {
    values.iter().any(|value| {
        value
            .as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
    })
}
