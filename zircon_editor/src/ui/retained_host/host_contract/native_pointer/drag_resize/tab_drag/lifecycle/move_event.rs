mod active;
mod start;

use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use self::active::dispatch_active_tab_drag_move;
use self::start::start_tab_drag_move;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_tab_drag_move(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let host = ui.global::<UiHostContext>();
    let (drag_active, pointer_x, pointer_y) = host.drag_pointer_snapshot()?;
    if pointer_x == x && pointer_y == y {
        return Some(NativePointerDispatchResult::idle());
    }
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::Drag);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::Drag);

    if drag_active {
        return Some(dispatch_active_tab_drag_move(ui, x, y));
    }
    Some(start_tab_drag_move(ui, pointer_x, pointer_y, x, y))
}
