use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};
use zircon_runtime_interface::ui::dispatch::{UiInputModifiers, UiPointerId};
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::NativePointerButtonState;
use super::super::super::viewport_button::viewport_button_id;
use super::super::input::button_dispatch_input;
use super::super::release_capture::finish_primary_capture_if_released;
use super::steps::dispatch_button_steps;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_pointer_button(
    ui: &UiHostWindow,
    pointer_id: UiPointerId,
    state: NativePointerButtonState,
    button: Option<UiPointerButton>,
    modifiers: UiInputModifiers,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::Click);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::Click);

    let button = button.unwrap_or(UiPointerButton::Primary);
    let Some(button_id) = viewport_button_id(button) else {
        return NativePointerDispatchResult::idle();
    };
    let press_release =
        if state == NativePointerButtonState::Released && button == UiPointerButton::Primary {
            ui.clear_template_button_press()
        } else {
            NativePointerDispatchResult::idle()
        };
    if let Some(result) = finish_primary_capture_if_released(ui, pointer_id, state, button, x, y) {
        return result.merge(press_release);
    }
    let input = button_dispatch_input(ui, pointer_id, button, button_id, modifiers);
    dispatch_button_steps(ui, state, input, x, y).merge(press_release)
}
