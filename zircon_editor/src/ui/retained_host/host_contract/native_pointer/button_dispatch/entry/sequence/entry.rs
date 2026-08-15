use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::super::super::super::NativePointerButtonState;
use super::super::input::button_dispatch_input;
use super::steps::dispatch_button_steps;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_pointer_button(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    button: Option<UiPointerButton>,
    modifiers: UiInputModifiers,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::Click);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::Click);

    let Some(input) = button_dispatch_input(ui, button, modifiers) else {
        return NativePointerDispatchResult::idle();
    };
    dispatch_button_steps(ui, state, input, x, y)
}
