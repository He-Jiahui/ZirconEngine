mod hover;

use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use self::hover::set_hovered_workbench_template_hit;
use super::super::redraw_result::workbench_template_node_move_redraw;
use super::super::routing::route_pointer_to_workbench_window;

pub(super) fn dispatch_workbench_template_pointer_move(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let hit = route_pointer_to_workbench_window(presentation, x, y)?;
    Some(dispatch_workbench_template_hit(ui, &hit))
}

pub(super) fn dispatch_workbench_template_popup_pointer_move(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let hit = route_pointer_to_workbench_window(presentation, x, y)?;
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
    let before = ui.get_pane_interaction_state();
    set_hovered_workbench_template_hit(ui, hit);
    workbench_template_node_move_redraw(hit, &before, &ui.get_pane_interaction_state())
}
