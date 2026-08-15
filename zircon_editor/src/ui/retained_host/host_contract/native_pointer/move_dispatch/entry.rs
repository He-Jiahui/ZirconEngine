mod body;
mod capture;

use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use self::body::dispatch_pointer_move_body;
use self::capture::dispatch_pointer_move_capture;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_pointer_move(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::IdleHover);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::IdleHover);

    if let Some(result) = dispatch_pointer_move_capture(ui, x, y) {
        return result;
    }

    dispatch_pointer_move_body(ui, x, y)
}
