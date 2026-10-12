use crate::ui::retained_host::host_contract::data::{
    HostDockOverflowMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::host_dock_overflow_menu::{
    contains, host_dock_overflow_popup_frame_with_state, host_dock_overflow_row_hit_in_popup,
};
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) struct HostDockOverflowPointerMoveDispatch {
    pub consumed: bool,
    pub result: NativePointerDispatchResult,
}

pub(super) fn dispatch_host_dock_overflow_pointer_move(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    state: &HostDockOverflowMenuStateData,
    x: f32,
    y: f32,
) -> Option<HostDockOverflowPointerMoveDispatch> {
    if !state.open {
        return None;
    }
    let Some(popup) = host_dock_overflow_popup_frame_with_state(presentation, state) else {
        clear_stale_hover(ui, state);
        return Some(HostDockOverflowPointerMoveDispatch {
            consumed: false,
            result: NativePointerDispatchResult::idle(),
        });
    };
    let consumed = contains(&popup, x, y);
    let hovered_tab_index = consumed
        .then(|| host_dock_overflow_row_hit_in_popup(presentation, &popup, state, x, y))
        .flatten()
        .map(|hit| hit.tab_index as i32)
        .unwrap_or(-1);
    if hovered_tab_index == state.hovered_tab_index {
        return Some(HostDockOverflowPointerMoveDispatch {
            consumed,
            result: NativePointerDispatchResult::idle(),
        });
    }
    ui.global::<UiHostContext>()
        .set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData {
            open: true,
            surface_key: state.surface_key.clone(),
            hovered_tab_index,
            scroll_offset: state.scroll_offset,
        });
    Some(HostDockOverflowPointerMoveDispatch {
        consumed,
        result: NativePointerDispatchResult::region(popup),
    })
}

fn clear_stale_hover(ui: &UiHostWindow, state: &HostDockOverflowMenuStateData) {
    if state.hovered_tab_index < 0 {
        return;
    }
    ui.global::<UiHostContext>()
        .set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData {
            open: true,
            surface_key: state.surface_key.clone(),
            hovered_tab_index: -1,
            scroll_offset: state.scroll_offset,
        });
}
