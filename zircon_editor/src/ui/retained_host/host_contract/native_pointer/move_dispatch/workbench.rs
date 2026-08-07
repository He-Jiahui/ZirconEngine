mod hover;

use crate::ui::retained_host::host_contract::data::HostPresentationGeneration;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use self::hover::set_hovered_workbench_template_hit;
use super::super::redraw_result::workbench_template_node_move_redraw;
use super::super::routing::route_pointer_to_workbench_generation;

pub(super) fn dispatch_workbench_template_pointer_move(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let hit = route_pointer_to_workbench_generation(generation, x, y)?;
    Some(dispatch_workbench_template_hit(ui, &hit))
}

pub(super) fn dispatch_workbench_template_popup_pointer_move(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let hit = route_pointer_to_workbench_generation(generation, x, y)?;
    if !matches!(
        hit.dispatch_kind.as_str(),
        "workbench_option" | "workbench_menu_item"
    ) {
        return None;
    }
    Some(dispatch_workbench_template_hit(ui, &hit))
}

fn dispatch_workbench_template_hit(
    ui: &UiHostWindow,
    hit: &TemplateNodePointerHit,
) -> NativePointerDispatchResult {
    let before = ui.get_host_presentation_generation();
    set_hovered_workbench_template_hit(ui, hit);
    let after = ui.get_host_presentation_generation();
    workbench_template_node_move_redraw(
        hit,
        before.pane_interaction_state(),
        after.pane_interaction_state(),
    )
}
