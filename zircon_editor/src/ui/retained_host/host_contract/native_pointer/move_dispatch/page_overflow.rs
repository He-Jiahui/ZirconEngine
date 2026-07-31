use crate::ui::retained_host::host_contract::data::{
    HostPageOverflowMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::host_page_overflow_menu::{
    host_page_overflow_popup_frame, host_page_overflow_popup_frame_contains,
    host_page_overflow_row_hit_in_popup,
};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) struct HostPageOverflowPointerMoveDispatch {
    pub consumed: bool,
    pub result: NativePointerDispatchResult,
}

pub(super) fn dispatch_host_page_overflow_pointer_move(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<HostPageOverflowPointerMoveDispatch> {
    if !presentation.host_page_overflow_menu_state.open {
        return None;
    }
    let Some(popup) = host_page_overflow_popup_frame(presentation) else {
        clear_stale_hover(ui, presentation);
        return Some(HostPageOverflowPointerMoveDispatch {
            consumed: false,
            result: NativePointerDispatchResult::idle(),
        });
    };
    let consumed = host_page_overflow_popup_frame_contains(&popup, x, y);
    let hovered_page_index = consumed
        .then(|| host_page_overflow_row_hit_in_popup(presentation, &popup, x, y))
        .flatten()
        .map(|hit| hit.page_index as i32)
        .unwrap_or(-1);
    if hovered_page_index
        == presentation
            .host_page_overflow_menu_state
            .hovered_page_index
    {
        return Some(HostPageOverflowPointerMoveDispatch {
            consumed,
            result: NativePointerDispatchResult::idle(),
        });
    }

    ui.global::<UiHostContext>()
        .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData {
            open: true,
            hovered_page_index,
            scroll_offset: presentation.host_page_overflow_menu_state.scroll_offset,
        });
    Some(HostPageOverflowPointerMoveDispatch {
        consumed,
        result: NativePointerDispatchResult::region(popup),
    })
}

fn clear_stale_hover(ui: &UiHostWindow, presentation: &HostWindowPresentationData) {
    if presentation
        .host_page_overflow_menu_state
        .hovered_page_index
        < 0
    {
        return;
    }
    ui.global::<UiHostContext>()
        .set_host_page_overflow_menu_state(HostPageOverflowMenuStateData {
            open: true,
            hovered_page_index: -1,
            scroll_offset: presentation.host_page_overflow_menu_state.scroll_offset,
        });
}
