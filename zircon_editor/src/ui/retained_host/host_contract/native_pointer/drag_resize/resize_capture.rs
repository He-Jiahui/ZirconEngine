use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use super::super::super::data::HostResizeStateData;
use super::super::super::globals::UiHostContext;
use super::super::super::redraw::NativePointerDispatchResult;
use super::super::super::window::UiHostWindow;
use super::super::redraw_result::resize_pointer_redraw;
use super::super::{HOST_POINTER_DOWN, HOST_POINTER_MOVE, HOST_POINTER_UP};

pub(in crate::ui::retained_host::host_contract) fn arm_native_resize(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) {
    let host = ui.global::<UiHostContext>();
    host.set_resize_state(HostResizeStateData {
        resize_active: true,
        resize_pointer_x: x,
        resize_pointer_y: y,
        ..HostResizeStateData::default()
    });
    host.invoke_host_resize_pointer_event(HOST_POINTER_DOWN, x, y);
}

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
