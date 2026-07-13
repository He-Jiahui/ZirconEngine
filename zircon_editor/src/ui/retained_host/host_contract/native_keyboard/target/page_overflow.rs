use super::model::{PopupKeyboardRow, PopupKeyboardTarget};
use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::host_page_overflow_menu::{
    host_page_overflow_popup_frame, host_page_overflow_row_frame,
};

const HOST_PAGE_OVERFLOW_CONTROL_ID: &str = "HostPageOverflowMenu";
pub(in crate::ui::retained_host::host_contract) const HOST_PAGE_OVERFLOW_DISPATCH_KIND: &str =
    "host_page_overflow";

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_keyboard_target(
    presentation: &HostWindowPresentationData,
) -> Option<PopupKeyboardTarget> {
    let popup_frame = host_page_overflow_popup_frame(presentation)?;
    let hidden_indices = &presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices;
    let rows = hidden_indices
        .iter()
        .enumerate()
        .filter_map(|(row_index, page_index)| {
            let tab = presentation
                .host_scene_data
                .page_chrome
                .tabs
                .row_data(*page_index)?;
            Some(PopupKeyboardRow {
                action_id: page_index.to_string().into(),
                value_text: tab.title.clone(),
                identity: tab.id.clone(),
                search_text: tab.title.clone(),
                focused: presentation
                    .host_page_overflow_menu_state
                    .hovered_page_index
                    == *page_index as i32,
                selected: tab.active,
                source_index: Some(*page_index),
                frame: host_page_overflow_row_frame(&popup_frame, row_index),
            })
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return None;
    }

    let current_index = rows.iter().position(|row| row.focused || row.selected);
    let current_row = current_index.and_then(|index| rows.get(index).cloned());
    let current_frame = current_row
        .as_ref()
        .map(|row| row.frame.clone())
        .unwrap_or_else(|| popup_frame.clone());

    Some(PopupKeyboardTarget {
        control_id: HOST_PAGE_OVERFLOW_CONTROL_ID.into(),
        dispatch_kind: HOST_PAGE_OVERFLOW_DISPATCH_KIND.into(),
        rows,
        current_index: current_index.unwrap_or(0),
        current_row,
        current_frame,
        popup_frame,
    })
}
