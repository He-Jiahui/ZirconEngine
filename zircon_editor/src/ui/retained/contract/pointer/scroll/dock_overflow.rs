use crate::ui::retained_host::host_contract::data::{
    HostDockOverflowMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::host_dock_overflow_menu::{
    contains, host_dock_overflow_popup_frame_with_state, host_dock_overflow_row_hit_in_popup,
    host_dock_overflow_scroll_offset_for_delta,
};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn dispatch_host_dock_overflow_menu_scroll(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    state: &HostDockOverflowMenuStateData,
    x: f32,
    y: f32,
    delta: f32,
) -> Option<NativePointerDispatchResult> {
    let popup = host_dock_overflow_popup_frame_with_state(presentation, state)?;
    if !contains(&popup, x, y) {
        return None;
    }
    let scroll_offset =
        host_dock_overflow_scroll_offset_for_delta(presentation, &popup, state, delta);
    let mut next = state.clone();
    next.scroll_offset = scroll_offset;
    next.hovered_tab_index = host_dock_overflow_row_hit_in_popup(presentation, &popup, &next, x, y)
        .map(|hit| hit.tab_index as i32)
        .unwrap_or(-1);
    if &next == state {
        return Some(NativePointerDispatchResult::idle());
    }
    ui.global::<UiHostContext>()
        .set_host_dock_overflow_menu_state(next);
    Some(NativePointerDispatchResult::region(popup))
}
