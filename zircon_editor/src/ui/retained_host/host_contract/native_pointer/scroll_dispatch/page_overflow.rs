use crate::ui::retained_host::host_contract::data::{
    HostPageOverflowMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::host_page_overflow_menu::{
    host_page_overflow_popup_frame, host_page_overflow_popup_frame_contains,
    host_page_overflow_row_hit_in_popup_for_scroll, host_page_overflow_scroll_offset_for_delta,
};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn dispatch_host_page_overflow_menu_scroll(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    delta: f32,
) -> Option<NativePointerDispatchResult> {
    let popup = host_page_overflow_popup_frame(presentation)?;
    if !host_page_overflow_popup_frame_contains(&popup, x, y) {
        return None;
    }

    let scroll_offset = host_page_overflow_scroll_offset_for_delta(presentation, &popup, delta);
    let hovered_page_index =
        host_page_overflow_row_hit_in_popup_for_scroll(presentation, &popup, x, y, scroll_offset)
            .map(|hit| hit.page_index as i32)
            .unwrap_or(-1);
    let state_changed = (scroll_offset - presentation.host_page_overflow_menu_state.scroll_offset)
        .abs()
        > f32::EPSILON
        || hovered_page_index
            != presentation
                .host_page_overflow_menu_state
                .hovered_page_index;
    if state_changed {
        ui.global::<UiHostContext>()
            .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData {
                open: true,
                hovered_page_index,
                scroll_offset,
            });
    }

    // Consume boundary scroll too: a pointer over this popup must never scroll
    // the covered document pane after the popup has reached either extent.
    Some(NativePointerDispatchResult::region(popup))
}
