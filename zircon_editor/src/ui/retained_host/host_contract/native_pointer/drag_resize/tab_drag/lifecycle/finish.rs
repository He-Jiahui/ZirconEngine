use crate::ui::retained_host::host_contract::data::HostDragStateData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use super::super::super::super::redraw_result::tab_drag_release_redraw;
use super::super::super::super::HOST_POINTER_UP;

pub(in crate::ui::retained_host::host_contract) fn finish_native_tab_drag(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let host = ui.global::<UiHostContext>();
    let drag_state = host.get_drag_state();
    if drag_state.drag_tab_id.is_empty() {
        return None;
    }
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::Drag);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::Drag);
    if drag_state.drag_active {
        let presentation = ui.get_host_presentation_generation();
        host.invoke_host_drag_pointer_event(HOST_POINTER_UP, x, y);
        let release_drag_state = host.get_drag_state();
        let redraw = tab_drag_release_redraw(presentation.structure(), &release_drag_state);
        host.set_drag_state(HostDragStateData::default());
        return Some(redraw);
    }
    host.set_drag_state(HostDragStateData::default());
    Some(NativePointerDispatchResult::idle())
}
