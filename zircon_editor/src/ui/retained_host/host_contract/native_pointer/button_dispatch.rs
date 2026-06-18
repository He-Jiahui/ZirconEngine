mod chrome_press;
mod close_prompt_hit;
mod pane_callbacks;
mod text_focus;
mod viewport_button;

use crate::ui::retained_host::host_contract::frame_geometry::{union_frame, union_optional_frames};
use crate::ui::retained_host::host_contract::globals::{PaneSurfaceHostContext, UiHostContext};
use crate::ui::retained_host::host_contract::native_popup_dismiss::dispatch_workbench_popup_outside_primary_press;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::template_activation_semantics::dispatch_template_node_primary_press;
use crate::ui::retained_host::host_contract::template_input_semantics::hit_is_text_input;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::host_contract::workbench_context_menu::workbench_context_menu_request_for_hit;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::close_prompt_damage::close_prompt_action_damage_frame;
use super::drag_resize::{arm_native_tab_drag, finish_native_resize, finish_native_tab_drag};
use super::menu_geometry::{
    menu_damage_frame, menu_damage_frame_with_state, menu_handles_point, menu_popup_handles_point,
};
use super::redraw_result::{chrome_press_redraw, resize_pointer_redraw};
use super::routing::{
    contains, route_pointer_to_pane, route_pointer_to_workbench_window, route_top_level_chrome,
    ChromePointerRoute,
};
use super::NativePointerButtonState;
use chrome_press::dispatch_chrome_press;
use close_prompt_hit::close_prompt_action_at;
use pane_callbacks::dispatch_pane_button;
use text_focus::focus_template_node_text_input;
use viewport_button::viewport_button_id;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_pointer_button(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    button: Option<UiPointerButton>,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::Click);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::Click);

    let button = button.unwrap_or(UiPointerButton::Primary);
    let presentation = ui.get_host_presentation();
    let Some(button_id) = viewport_button_id(button) else {
        return NativePointerDispatchResult::idle();
    };

    if state == NativePointerButtonState::Released && button == UiPointerButton::Primary {
        if let Some(result) = finish_native_resize(ui, x, y) {
            return result;
        }
        if let Some(result) = finish_native_tab_drag(ui, x, y) {
            return result;
        }
    }

    if let Some(action_id) = close_prompt_action_at(&presentation, x, y) {
        if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
            ui.global::<UiHostContext>()
                .invoke_close_prompt_action_clicked(action_id);
            return match close_prompt_action_damage_frame(&presentation) {
                Some(damage) => NativePointerDispatchResult::region_with_frame_update(damage),
                None => NativePointerDispatchResult::full_frame(),
            };
        }
        return NativePointerDispatchResult::idle();
    }
    if presentation.close_prompt.visible && contains(&presentation.close_prompt.overlay_frame, x, y)
    {
        return NativePointerDispatchResult::idle();
    }

    let cleared_text_input_frame =
        if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
            let host = ui.global::<UiHostContext>();
            let focus = host.get_text_input_focus();
            if focus.is_active() {
                let frame = focus.edit_frame.clone();
                host.clear_text_input_focus();
                Some(frame)
            } else {
                None
            }
        } else {
            None
        };

    if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
        if menu_handles_point(&presentation, x, y) || menu_popup_handles_point(&presentation, x, y)
        {
            let before_damage = menu_damage_frame(&presentation);
            ui.global::<UiHostContext>()
                .invoke_menu_pointer_clicked(x, y);
            let after_state = ui.get_menu_state();
            let after_damage = menu_damage_frame_with_state(&presentation, &after_state);
            let mut damage = union_frame(&before_damage, &after_damage);
            if let Some(frame) = cleared_text_input_frame.clone() {
                damage = union_frame(&damage, &frame);
            }
            return NativePointerDispatchResult::region(damage);
        }
    }

    if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
        if let Some(result) = dispatch_workbench_popup_outside_primary_press(
            ui,
            &presentation,
            x,
            y,
            cleared_text_input_frame.clone(),
        ) {
            return result;
        }
    }

    if let Some(route) = route_top_level_chrome(&presentation, x, y) {
        if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
            arm_native_tab_drag(ui, &presentation, &route, x, y);
            let redraw = if matches!(&route, ChromePointerRoute::Resize) {
                resize_pointer_redraw(&presentation, cleared_text_input_frame.clone())
            } else {
                chrome_press_redraw(&presentation, &route, cleared_text_input_frame.clone())
            };
            dispatch_chrome_press(ui, route, x, y);
            return redraw;
        }
    }

    if let Some(hit) = route_pointer_to_workbench_window(&presentation, x, y) {
        if state == NativePointerButtonState::Pressed && button == UiPointerButton::Secondary {
            let Some(request) = workbench_context_menu_request_for_hit(&hit, x, y) else {
                return NativePointerDispatchResult::idle();
            };
            ui.global::<PaneSurfaceHostContext>()
                .invoke_workbench_context_menu_requested(request);
            let damage =
                union_optional_frames(cleared_text_input_frame.clone(), Some(hit.frame.clone()))
                    .unwrap_or_else(|| hit.frame.clone());
            return NativePointerDispatchResult::region_with_frame_update(damage);
        }
        if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
            if hit_is_text_input(&hit) {
                if focus_template_node_text_input(ui, &hit) {
                    let damage = union_optional_frames(
                        cleared_text_input_frame.clone(),
                        Some(hit.frame.clone()),
                    )
                    .unwrap_or_else(|| hit.frame.clone());
                    return NativePointerDispatchResult::region(damage);
                }
                return NativePointerDispatchResult::idle();
            }
            let pane_host = ui.global::<PaneSurfaceHostContext>();
            dispatch_template_node_primary_press(&pane_host, hit.clone());
            let damage =
                union_optional_frames(cleared_text_input_frame.clone(), Some(hit.frame.clone()))
                    .unwrap_or_else(|| hit.frame.clone());
            return NativePointerDispatchResult::region_with_frame_update(damage);
        }
        if state == NativePointerButtonState::Released {
            return NativePointerDispatchResult::region(hit.frame);
        }
    }

    if let Some(pointer) = route_pointer_to_pane(&presentation, x, y) {
        return dispatch_pane_button(
            ui,
            &presentation,
            pointer,
            state,
            button,
            button_id,
            cleared_text_input_frame,
        );
    }

    if let Some(frame) = cleared_text_input_frame {
        return NativePointerDispatchResult::region(frame);
    }
    NativePointerDispatchResult::idle()
}
