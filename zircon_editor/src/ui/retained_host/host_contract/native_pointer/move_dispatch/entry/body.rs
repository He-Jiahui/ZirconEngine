use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::routing::route_top_level_chrome;
use super::super::super::{tooltip_target_for_chrome_route, WorkbenchTooltipPointerTarget};
use super::super::clear::clear_hovered_template_move;
use super::super::dock_overflow::dispatch_host_dock_overflow_pointer_move;
use super::super::menu::dispatch_menu_pointer_move;
use super::super::page_overflow::dispatch_host_page_overflow_pointer_move;
use super::super::pane::dispatch_pane_pointer_move;
use super::super::workbench::{
    dispatch_workbench_template_hit, is_workbench_template_popup_hit,
    workbench_template_pointer_hit,
};

pub(super) fn dispatch_pointer_move_body(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> (
    NativePointerDispatchResult,
    Option<WorkbenchTooltipPointerTarget>,
) {
    let generation = ui.get_host_presentation_generation();
    let structure = generation.structure();
    let dock_overflow = if generation.dock_overflow_menu_state().open {
        dispatch_host_dock_overflow_pointer_move(
            ui,
            structure,
            generation.dock_overflow_menu_state(),
            x,
            y,
        )
    } else {
        None
    };
    let dock_overflow = match dock_overflow {
        Some(dispatch) if dispatch.consumed => return (dispatch.result, None),
        other => other,
    };
    let overflow = if generation.page_overflow_menu_state().open {
        dispatch_host_page_overflow_pointer_move(
            ui,
            structure,
            generation.page_overflow_menu_state(),
            x,
            y,
        )
    } else {
        None
    };
    let overflow = match overflow {
        Some(dispatch) if dispatch.consumed => return (dispatch.result, None),
        other => other,
    };
    let (routed, tooltip_target) =
        if let Some(result) = dispatch_menu_pointer_move(ui, &generation, x, y) {
            (result, None)
        } else {
            let workbench_hit = workbench_template_pointer_hit(&generation, x, y);
            if let Some(hit) = workbench_hit
                .as_ref()
                .filter(|hit| is_workbench_template_popup_hit(hit))
            {
                let result = if hit.dispatchable {
                    dispatch_workbench_template_hit(ui, &generation, hit)
                } else {
                    clear_hovered_template_move(ui)
                };
                (
                    result,
                    hit.surface_node_id
                        .map(WorkbenchTooltipPointerTarget::SurfaceNode),
                )
            } else {
                let mut host_chrome_target = route_top_level_chrome(structure, x, y)
                    .as_ref()
                    .and_then(|route| tooltip_target_for_chrome_route(structure, route))
                    .map(WorkbenchTooltipPointerTarget::HostChrome);
                if let Some(result) = dispatch_pane_pointer_move(ui, &generation, x, y) {
                    (result, host_chrome_target.take())
                } else if let Some(hit) = workbench_hit.as_ref() {
                    let result = if hit.dispatchable {
                        dispatch_workbench_template_hit(ui, &generation, hit)
                    } else {
                        clear_hovered_template_move(ui)
                    };
                    (
                        result,
                        host_chrome_target.take().or_else(|| {
                            hit.surface_node_id
                                .map(WorkbenchTooltipPointerTarget::SurfaceNode)
                        }),
                    )
                } else {
                    (clear_hovered_template_move(ui), host_chrome_target.take())
                }
            }
        };
    let result = match overflow {
        Some(dispatch) => dispatch.result.merge(routed),
        None => routed,
    };
    let result = match dock_overflow {
        Some(dispatch) => dispatch.result.merge(result),
        None => result,
    };
    (result, tooltip_target)
}
