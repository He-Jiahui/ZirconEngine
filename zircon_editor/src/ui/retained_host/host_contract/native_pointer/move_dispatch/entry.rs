mod body;
mod capture;

use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};
use zircon_runtime_interface::ui::dispatch::UiPointerId;

use self::body::dispatch_pointer_move_body;
use self::capture::dispatch_pointer_move_capture;
use super::super::WorkbenchTooltipPointerTarget;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_pointer_move(
    ui: &UiHostWindow,
    pointer_id: Option<UiPointerId>,
    x: f32,
    y: f32,
) -> (
    NativePointerDispatchResult,
    Option<WorkbenchTooltipPointerTarget>,
) {
    if let Some(result) = pointer_id.and_then(|id| dispatch_pointer_move_capture(ui, id, x, y)) {
        return (result, None);
    }

    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::IdleHover);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::IdleHover);

    dispatch_pointer_move_body(ui, x, y)
}
