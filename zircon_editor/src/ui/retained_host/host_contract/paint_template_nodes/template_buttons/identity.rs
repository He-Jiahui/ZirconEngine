use super::super::super::data::TemplatePaneNodeData;
use super::super::super::template_component_family::{
    TemplateComponentFamily, is_component_family, uses_workbench_visual_language,
};
use super::super::style_selector::{WorkbenchButtonKind, is_tab_like_workbench_button};

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
    let key = button_key(node);
    if key.contains("danger") || key.contains("delete") || key.contains("trash") {
        WorkbenchButtonKind::Danger
    } else if key.contains("primary") || key.contains("filled") || key.contains("accent") {
        WorkbenchButtonKind::Primary
    } else if key.contains("tertiary") || key.contains("text") {
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_key(
    node: &TemplatePaneNodeData,
) -> String {
    format!(
        "{} {} {} {} {} {}",
        node.control_id.as_str(),
        node.text.as_str(),
        node.value_text.as_str(),
        node.button_variant.as_str(),
        node.surface_variant.as_str(),
        node.validation_level.as_str()
    )
    .to_ascii_lowercase()
}
