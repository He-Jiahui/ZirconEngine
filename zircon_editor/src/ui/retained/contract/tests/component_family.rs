use super::*;
use crate::ui::retained_host::host_contract::TemplatePaneNodeData;

#[test]
fn component_family_prefers_declared_component_role() {
    let node = node_with_contract("AnyControl", "input", "button", "leaf");

    assert_eq!(
        template_component_family(&node),
        Some(TemplateComponentFamily::Button)
    );
}

#[test]
fn component_family_uses_category_and_layout_for_collections() {
    let grid = node_with_contract("AnyTable", "collection", "", "grid");
    let list = node_with_contract("AnyList", "collection", "", "virtual-list");

    assert_eq!(
        template_component_family(&grid),
        Some(TemplateComponentFamily::Table)
    );
    assert_eq!(
        template_component_family(&list),
        Some(TemplateComponentFamily::List)
    );
}

#[test]
fn workbench_visual_language_can_be_declared_without_control_prefix() {
    let mut node = node_with_contract("Primary", "input", "button", "leaf");
    node.component_variant = "workbench-button".into();

    assert!(uses_workbench_visual_language(&node));
    assert!(is_component_family(&node, TemplateComponentFamily::Button));
}

#[test]
fn range_field_is_a_slider_family() {
    let range = node_with_contract("AnyRange", "input", "range-field", "leaf");
    let by_id = node_with_contract("WorkbenchInputSlider", "", "", "");

    assert_eq!(
        template_component_family(&range),
        Some(TemplateComponentFamily::Slider)
    );
    assert_eq!(
        template_component_family(&by_id),
        Some(TemplateComponentFamily::Slider)
    );
}

fn node_with_contract(
    control_id: &str,
    category: &str,
    role: &str,
    layout_role: &str,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        component_category: category.into(),
        component_role: role.into(),
        component_layout_role: layout_role.into(),
        ..TemplatePaneNodeData::default()
    }
}
