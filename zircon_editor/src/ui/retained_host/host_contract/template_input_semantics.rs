use super::surface_hit_test::TemplateNodePointerHit;
use super::template_component_family::TemplateComponentFamily;
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn hit_is_text_input(hit: &TemplateNodePointerHit) -> bool {
    if matches!(
        hit.dispatch_kind.as_str(),
        "workbench_option" | "workbench_menu_item"
    ) {
        return false;
    }
    hit.dispatch_kind.as_str() == "welcome_text"
        || hit.component_family == Some(TemplateComponentFamily::TextInput)
        || matches!(hit.component_role.as_str(), "input-field" | "number-field")
}

pub(super) fn text_input_edit_target_id(hit: &TemplateNodePointerHit) -> SharedString {
    if !hit.edit_action_id.is_empty() {
        hit.edit_action_id.clone()
    } else if hit.dispatch_kind.as_str() == "welcome_text" && !hit.action_id.is_empty() {
        hit.action_id.clone()
    } else if hit_uses_component_text_input_semantics(hit) && !hit.binding_id.is_empty() {
        hit.binding_id.clone()
    } else {
        SharedString::new()
    }
}

fn hit_uses_component_text_input_semantics(hit: &TemplateNodePointerHit) -> bool {
    hit.component_family == Some(TemplateComponentFamily::TextInput)
        || matches!(hit.component_role.as_str(), "input-field" | "number-field")
}

#[cfg(test)]
mod tests {
    use super::super::data::FrameRect;
    use super::*;

    #[test]
    fn text_input_family_uses_binding_as_edit_target() {
        let mut hit = hit_with_family(Some(TemplateComponentFamily::TextInput));
        hit.component_role = "text-field".into();
        hit.binding_id = "TextField.Binding".into();

        assert!(hit_is_text_input(&hit));
        assert_eq!(
            text_input_edit_target_id(&hit).as_str(),
            "TextField.Binding"
        );
    }

    #[test]
    fn popup_rows_do_not_inherit_text_input_focus() {
        let mut hit = hit_with_family(Some(TemplateComponentFamily::Dropdown));
        hit.dispatch_kind = "workbench_option".into();
        hit.binding_id = "Dropdown.Binding".into();

        assert!(!hit_is_text_input(&hit));
        assert_eq!(text_input_edit_target_id(&hit).as_str(), "");
    }

    fn hit_with_family(family: Option<TemplateComponentFamily>) -> TemplateNodePointerHit {
        TemplateNodePointerHit {
            control_id: "Control".into(),
            action_id: SharedString::new(),
            binding_id: SharedString::new(),
            dispatch_kind: SharedString::new(),
            component_role: SharedString::new(),
            component_family: family,
            value_text: SharedString::new(),
            edit_action_id: SharedString::new(),
            commit_action_id: SharedString::new(),
            frame: FrameRect::default(),
        }
    }
}
