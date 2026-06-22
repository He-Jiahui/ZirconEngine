use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::template_input_semantics::hit_is_text_input;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::text_focus::focus_template_node_text_input;
use super::damage::hit_damage;

pub(super) fn dispatch_workbench_text_input_primary_button(
    ui: &UiHostWindow,
    hit: &TemplateNodePointerHit,
    cleared_text_input_frame: Option<&FrameRect>,
) -> Option<NativePointerDispatchResult> {
    if !hit_is_text_input(hit) {
        return None;
    }
    if focus_template_node_text_input(ui, hit) {
        let damage = hit_damage(cleared_text_input_frame, hit);
        return Some(NativePointerDispatchResult::region(damage));
    }
    Some(NativePointerDispatchResult::idle())
}
