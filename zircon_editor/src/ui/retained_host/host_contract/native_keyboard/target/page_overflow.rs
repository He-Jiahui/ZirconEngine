use super::model::{PopupKeyboardRow, PopupKeyboardTarget};
use crate::ui::retained_host::host_contract::data::{
    HostPageOverflowMenuStateData, HostWindowPresentationData,
};
use crate::ui::retained_host::host_contract::host_page_overflow_menu::{
    host_page_overflow_popup_frame_with_state, host_page_overflow_row_frame_with_state,
};

const HOST_PAGE_OVERFLOW_CONTROL_ID: &str = "HostPageOverflowMenu";
pub(in crate::ui::retained_host::host_contract) const HOST_PAGE_OVERFLOW_DISPATCH_KIND: &str =
    "host_page_overflow";

#[cfg(test)]
#[path = "page_overflow/tests/focus_index_single_scan_tests.rs"]
mod focus_index_single_scan_tests;

fn record_first_current_index(
    current_index: &mut Option<usize>,
    row_index: usize,
    focused: bool,
    selected: bool,
) {
    if current_index.is_none() && (focused || selected) {
        *current_index = Some(row_index);
    }
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_keyboard_target(
    presentation: &HostWindowPresentationData,
) -> Option<PopupKeyboardTarget> {
    host_page_overflow_keyboard_target_with_state(
        presentation,
        &presentation.host_page_overflow_menu_state,
    )
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_keyboard_target_with_state(
    presentation: &HostWindowPresentationData,
    state: &HostPageOverflowMenuStateData,
) -> Option<PopupKeyboardTarget> {
    let popup_frame = host_page_overflow_popup_frame_with_state(presentation, state)?;
    let hidden_indices = &presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices;
    let mut rows = Vec::with_capacity(hidden_indices.len());
    let mut current_index = None;
    for (row_index, page_index) in hidden_indices.iter().enumerate() {
        let Some(tab) = presentation
            .host_scene_data
            .page_chrome
            .tabs
            .row_data(*page_index)
        else {
            continue;
        };
        let focused = state.hovered_page_index == *page_index as i32;
        let selected = tab.active;
        record_first_current_index(&mut current_index, rows.len(), focused, selected);
        rows.push(PopupKeyboardRow {
            action_id: page_index.to_string().into(),
            value_text: tab.title.clone(),
            identity: tab.id.clone(),
            search_text: tab.title.clone(),
            focused,
            selected,
            source_index: Some(*page_index),
            frame: host_page_overflow_row_frame_with_state(
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

    let current_row = current_index.and_then(|index| rows.get(index).cloned());
    let current_frame = current_row
        .as_ref()
        .map(|row| row.frame.clone())
        .unwrap_or_else(|| popup_frame.clone());
    let total_count = rows.len();

    Some(PopupKeyboardTarget {
        control_id: HOST_PAGE_OVERFLOW_CONTROL_ID.into(),
        dispatch_kind: HOST_PAGE_OVERFLOW_DISPATCH_KIND.into(),
        rows,
        current_index: current_index.unwrap_or(0),
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

#[cfg(test)]
#[path = "tests/page_overflow.rs"]
mod tests;
