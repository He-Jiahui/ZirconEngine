use super::super::surface_hit_test::TemplateNodePointerHit;
use super::super::template_component_family::TemplateComponentFamily;

pub(in crate::ui::retained_host::host_contract) fn hit_is_text_input(
    hit: &TemplateNodePointerHit,
) -> bool {
    if matches!(
        hit.dispatch_kind.as_str(),
        "workbench_option" | "workbench_menu_item"
    ) {
        return false;
    }
    hit.dispatch_kind.as_str() == "welcome_text" || hit_uses_component_text_input_semantics(hit)
}

pub(in crate::ui::retained_host::host_contract) fn hit_uses_component_text_input_semantics(
    hit: &TemplateNodePointerHit,
) -> bool {
    hit.component_family == Some(TemplateComponentFamily::TextInput)
        || matches!(hit.component_role.as_str(), "input-field" | "number-field")
}
