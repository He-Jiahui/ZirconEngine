use super::drag_resize::{dispatch_native_resize_move, dispatch_native_tab_drag_move};
use super::menu_geometry::{menu_damage_frame, menu_handles_point, menu_popup_handles_point};
use super::redraw_result::{pointer_move_redraw, workbench_template_node_move_redraw};
use super::routing::{
    route_pointer_move_to_pane, route_pointer_to_workbench_window, PanePointerTarget,
};
use super::template_hover_damage::template_hover_damage;
use super::{VIEWPORT_POINTER_BUTTON_NONE, VIEWPORT_POINTER_MOVE};
use crate::ui::retained_host::host_contract::globals::{PaneSurfaceHostContext, UiHostContext};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_pointer_move(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::IdleHover);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::IdleHover);

    if let Some(result) = dispatch_native_resize_move(ui, x, y) {
        return result;
    }

    if let Some(result) = dispatch_native_tab_drag_move(ui, x, y) {
        return result;
    }

    let presentation = ui.get_host_presentation();
    if menu_handles_point(&presentation, x, y) || menu_popup_handles_point(&presentation, x, y) {
        let before = ui.get_menu_state();
        ui.global::<UiHostContext>().invoke_menu_pointer_moved(x, y);
        if before == ui.get_menu_state() {
            return NativePointerDispatchResult::idle();
        }
        return NativePointerDispatchResult::region(menu_damage_frame(&presentation));
    }

    if let Some(hit) = route_pointer_to_workbench_window(&presentation, x, y) {
        let before = ui.get_pane_interaction_state();
        set_hovered_workbench_template_hit(ui, &hit);
        return workbench_template_node_move_redraw(
            &hit,
            &before,
            &ui.get_pane_interaction_state(),
        );
    }

    if let Some(pointer) = route_pointer_move_to_pane(&presentation, x, y) {
        let before = ui.get_pane_interaction_state();
        let pane_host = ui.global::<PaneSurfaceHostContext>();
        match &pointer.target {
            PanePointerTarget::Hierarchy => pane_host.invoke_hierarchy_pointer_moved(
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::Welcome => pane_host.invoke_welcome_recent_pointer_moved(
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetTree(mode) => pane_host.invoke_asset_tree_pointer_moved(
                mode.clone(),
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetContent(mode) => pane_host.invoke_asset_content_pointer_moved(
                mode.clone(),
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetReference(mode, list_kind) => pane_host
                .invoke_asset_reference_pointer_moved(
                    mode.clone(),
                    list_kind.clone(),
                    pointer.local_x,
                    pointer.local_y,
                    pointer.width,
                    pointer.height,
                ),
            PanePointerTarget::TemplateNode(hit) => ui.set_hovered_template_node_for_pointer_move(
                hit.control_id.clone(),
                hit.frame.clone(),
            ),
            PanePointerTarget::Viewport(_) => {
                ui.clear_hovered_template_node_for_pointer_move();
                pane_host.invoke_viewport_pointer_event(
                    VIEWPORT_POINTER_MOVE,
                    VIEWPORT_POINTER_BUTTON_NONE,
                    pointer.local_x,
                    pointer.local_y,
                    0.0,
                )
            }
            PanePointerTarget::Console
            | PanePointerTarget::Inspector
            | PanePointerTarget::BrowserAssetDetails
            | PanePointerTarget::ViewportToolbar(_)
            | PanePointerTarget::UiAsset
            | PanePointerTarget::Other => {
                ui.clear_hovered_template_node_for_pointer_move();
            }
        }
        return pointer_move_redraw(&pointer, &before, &ui.get_pane_interaction_state());
    }

    let before = ui.get_pane_interaction_state();
    ui.clear_hovered_template_node_for_pointer_move();
    if let Some(damage) = template_hover_damage(&before, &ui.get_pane_interaction_state()) {
        return NativePointerDispatchResult::region(damage);
    }
    NativePointerDispatchResult::idle()
}

fn set_hovered_workbench_template_hit(ui: &UiHostWindow, hit: &TemplateNodePointerHit) {
    if matches!(
        hit.dispatch_kind.as_str(),
        "workbench_option" | "workbench_menu_item"
    ) {
        ui.set_hovered_template_row_for_pointer_move(
            hit.control_id.clone(),
            hit.dispatch_kind.clone(),
            hit.action_id.clone(),
            hit.value_text.clone(),
            hit.frame.clone(),
        );
    } else {
        ui.set_hovered_template_node_for_pointer_move(hit.control_id.clone(), hit.frame.clone());
    }
}
