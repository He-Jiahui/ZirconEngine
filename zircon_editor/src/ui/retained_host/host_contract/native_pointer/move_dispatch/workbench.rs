mod hover;

use crate::ui::retained_host::host_contract::data::HostPresentationGeneration;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerMoveHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use self::hover::set_hovered_workbench_template_hit;
use super::super::redraw_result::workbench_template_node_move_redraw;
use super::super::routing::route_pointer_move_to_workbench_generation;

pub(super) fn workbench_template_pointer_hit(
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerMoveHit<'_>> {
    route_pointer_move_to_workbench_generation(generation, x, y)
}

pub(super) fn is_workbench_template_popup_hit(hit: &TemplateNodePointerMoveHit<'_>) -> bool {
    hit.kind.is_popup()
}

pub(super) fn dispatch_workbench_template_hit(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    hit: &TemplateNodePointerMoveHit<'_>,
) -> NativePointerDispatchResult {
    let before = generation.pane_interaction_state();
    set_hovered_workbench_template_hit(ui, hit);
    let after = ui.get_pane_interaction_generation();
    workbench_template_node_move_redraw(&hit.frame, before, after.as_ref())
}
