use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use super::super::super::super::globals::UiHostContext;
use super::super::super::super::redraw::NativePointerDispatchResult;
use super::super::super::super::window::UiHostWindow;
use super::super::super::redraw_result::resize_pointer_redraw;
use super::super::super::HOST_POINTER_UP;

pub(in crate::ui::retained_host::host_contract) fn finish_native_resize(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let host = ui.global::<UiHostContext>();
    if !host.get_resize_state().resize_active {
        return None;
    }
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::DrawerResize);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::DrawerResize);

    let presentation = ui.get_host_presentation();
    host.invoke_host_resize_pointer_event(HOST_POINTER_UP, x, y);
    host.clear_resize_state();
    Some(resize_pointer_redraw(&presentation, None))
}
