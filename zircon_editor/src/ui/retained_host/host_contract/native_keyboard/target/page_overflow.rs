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
                frame: host_page_overflow_row_frame(presentation, &popup_frame, row_index),
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
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::ui::retained_host::host_contract::data::{
        FrameRect, HostPageOverflowMenuStateData, HostWindowPresentationData, TabData,
    };
    use crate::ui::retained_host::host_contract::native_keyboard::WorkbenchPopupKeyboardCommand;
    use crate::ui::retained_host::primitives::{ModelRc, VecModel};

    #[test]
    fn host_page_overflow_target_stays_local_and_never_requests_a_page_window() {
        let target = host_page_overflow_keyboard_target(&overflow_presentation())
            .expect("open host-page overflow should expose its keyboard target");

        assert_eq!(target.window_offset, 0);
        assert_eq!(target.window_count, target.rows.len());
        assert_eq!(target.total_count, target.rows.len());
        assert!(!target.window_navigation_enabled);
        assert!(target.window_query.is_empty());
        assert!(
            target
                .next_move(WorkbenchPopupKeyboardCommand::PageDown)
                .is_none()
        );
        assert!(
            target
                .next_move(WorkbenchPopupKeyboardCommand::PageUp)
                .is_none()
        );
    }

    fn overflow_presentation() -> HostWindowPresentationData {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.page_chrome.overflow_frame = FrameRect {
            x: 188.0,
            y: 29.0,
            width: 34.0,
            height: 28.0,
        };
        presentation.host_scene_data.page_chrome.tabs = model_rc(vec![
            tab("workbench", "Workbench", true),
            tab("assets", "Assets", false),
            tab("animation", "Animation", false),
            tab("tags", "Tags", false),
        ]);
        presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices = vec![1, 2, 3];
        presentation.host_page_overflow_menu_state = HostPageOverflowMenuStateData {
            open: true,
            hovered_page_index: 1,
            scroll_offset: 0.0,
        };
        presentation
    }

    fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
        ModelRc::from(Rc::new(VecModel::from(rows)))
    }

    fn tab(id: &str, title: &str, active: bool) -> TabData {
        TabData {
            id: id.into(),
            title: title.into(),
            active,
            ..TabData::default()
        }
    }
}
