use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use super::super::super::super::globals::UiHostContext;
use super::super::super::super::redraw::NativePointerDispatchResult;
use super::super::super::super::window::UiHostWindow;
use super::super::super::redraw_result::resize_pointer_redraw;
use super::super::super::HOST_POINTER_MOVE;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_resize_move(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let host = ui.global::<UiHostContext>();
    match host.update_resize_pointer_if_active(x, y) {
        None => return None,
        Some(false) => return Some(NativePointerDispatchResult::idle()),
        Some(true) => {}
    }
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::DrawerResize);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::DrawerResize);

    let presentation = ui.get_host_presentation_generation();
    host.invoke_host_resize_pointer_event(HOST_POINTER_MOVE, x, y);
    Some(resize_pointer_redraw(presentation.structure(), None))
}
