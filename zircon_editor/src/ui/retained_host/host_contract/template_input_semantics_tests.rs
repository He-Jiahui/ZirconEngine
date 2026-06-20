use super::super::data::FrameRect;
use super::super::surface_hit_test::TemplateNodePointerHit;
use super::super::template_component_family::TemplateComponentFamily;
use super::{hit_is_text_input, text_input_edit_target_id};
use crate::ui::retained_host::primitives::SharedString;

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
