use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::template_activation_semantics::dispatch_template_node_primary_press;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::damage::hit_damage;

pub(super) fn dispatch_workbench_template_primary_button(
    ui: &UiHostWindow,
    hit: TemplateNodePointerHit,
    cleared_text_input_frame: Option<&FrameRect>,
) -> NativePointerDispatchResult {
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    if std::env::var_os("ZIRCON_TRACE_EDITOR_INTERACTION").is_some() {
        eprintln!("editor_interaction kind=workbench_press control={} action={} binding={} dispatch={} disabled={}", hit.control_id, hit.action_id, hit.binding_id, hit.dispatch_kind, hit.disabled);
    }
    let damage = hit_damage(cleared_text_input_frame, &hit);
    dispatch_template_node_primary_press(&pane_host, hit);
    NativePointerDispatchResult::region_with_frame_update(damage)
}
