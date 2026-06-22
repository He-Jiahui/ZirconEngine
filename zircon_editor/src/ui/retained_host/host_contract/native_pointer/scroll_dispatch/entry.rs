use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

use super::menu::dispatch_menu_pointer_scroll;
use super::pane::dispatch_pane_pointer_scroll;

pub(in crate::ui::retained_host::host_contract) fn dispatch_native_pointer_scroll(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
    delta: f32,
) -> NativePointerDispatchResult {
    let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::IdleHover);
    let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::IdleHover);

    let presentation = ui.get_host_presentation();
    if let Some(result) = dispatch_menu_pointer_scroll(ui, &presentation, x, y, delta) {
        return result;
    }
    if let Some(result) = dispatch_pane_pointer_scroll(ui, &presentation, x, y, delta) {
        return result;
    }
    NativePointerDispatchResult::idle()
}
