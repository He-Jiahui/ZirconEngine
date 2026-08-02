use crate::ui::retained_host::ui_perf::{
    UiPerfScenario, enter_ui_perf_scenario, time_ui_perf_scenario,
};

use super::super::super::super::globals::UiHostContext;
use super::super::super::super::redraw::NativePointerDispatchResult;
use super::super::super::super::window::UiHostWindow;
use super::super::super::HOST_POINTER_MOVE;
use super::super::super::redraw_result::resize_pointer_redraw;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_resize_move(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let host = ui.global::<UiHostContext>();
    let mut resize_state = host.get_resize_state();
    if !resize_state.resize_active {
        return None;
    }
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::DrawerResize);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::DrawerResize);

    resize_state.resize_pointer_x = x;
    resize_state.resize_pointer_y = y;
    host.set_resize_state(resize_state);
    let presentation = ui.get_host_presentation();
    host.invoke_host_resize_pointer_event(HOST_POINTER_MOVE, x, y);
    Some(resize_pointer_redraw(&presentation, None))
}
