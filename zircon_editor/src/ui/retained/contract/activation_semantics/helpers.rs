use super::super::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract) fn action_or_control_id(
    hit: &TemplateNodePointerHit,
) -> SharedString {
    if hit.action_id.is_empty() {
        hit.control_id.clone()
    } else {
        hit.action_id.clone()
    }
}
