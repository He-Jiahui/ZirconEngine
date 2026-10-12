use super::model::{PopupKeyboardRow, PopupKeyboardTarget};
use crate::ui::retained_host::host_contract::data::{
    HostDockOverflowMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::host_dock_overflow_menu::{
    host_dock_overflow_hidden_indices, host_dock_overflow_popup_frame_with_state,
    host_dock_overflow_projection, host_dock_overflow_row_frame_with_state,
};

pub(in crate::ui::retained_host::host_contract) const HOST_DOCK_OVERFLOW_DISPATCH_KIND: &str =
    "host_dock_overflow";

pub(super) fn host_dock_overflow_keyboard_target_with_state(
    presentation: &HostWindowPresentationData,
    state: &HostDockOverflowMenuStateData,
) -> Option<PopupKeyboardTarget> {
    let popup_frame = host_dock_overflow_popup_frame_with_state(presentation, state)?;
    let projection = host_dock_overflow_projection(presentation, state)?;
    let hidden_indices = host_dock_overflow_hidden_indices(&projection);
    let mut rows = Vec::with_capacity(hidden_indices.len());
    let mut current_index = None;
    for (row_index, tab_index) in hidden_indices.iter().copied().enumerate() {
        let Some(tab) = projection.tabs.get(tab_index) else {
            continue;
        };
        let focused = state.hovered_tab_index == tab_index as i32;
        if current_index.is_none() && (focused || tab.active) {
            current_index = Some(rows.len());
        }
        rows.push(PopupKeyboardRow {
            action_id: tab_index.to_string().into(),
            value_text: tab.title.clone(),
            identity: tab.id.clone(),
            search_text: tab.title.clone(),
            focused,
            selected: tab.active,
            source_index: Some(tab_index),
            frame: host_dock_overflow_row_frame_with_state(
                presentation,
                &popup_frame,
                row_index,
                state,
            ),
        });
    }
    if rows.is_empty() {
        return None;
    }
    let current_index = current_index.unwrap_or(0);
    let current_row = rows.get(current_index).cloned();
    let current_frame = current_row
        .as_ref()
        .map(|row| row.frame.clone())
        .unwrap_or_else(|| popup_frame.clone());
    let total_count = rows.len();
    Some(PopupKeyboardTarget {
        control_id: format!("HostDockOverflowMenu:{}", projection.surface_key).into(),
        dispatch_kind: HOST_DOCK_OVERFLOW_DISPATCH_KIND.into(),
        rows,
        current_index,
        current_row,
        current_frame,
        popup_frame,
        window_offset: 0,
        window_count: total_count,
        total_count,
        window_navigation_enabled: false,
        window_query: "".into(),
    })
}
