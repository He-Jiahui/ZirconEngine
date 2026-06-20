use super::super::surface_hit_test::TemplateNodePointerHit;
use super::classification::hit_uses_component_text_input_semantics;
use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract) fn text_input_edit_target_id(
    hit: &TemplateNodePointerHit,
) -> SharedString {
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
