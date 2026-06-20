use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

use super::super::{
    action_matches, DEFAULT_PAGED_LIST_PAGE_SIZE, DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT,
};

pub(super) fn demo_list_input(action_id: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    match action_id {
        action if action_matches(action, "list_row_hovered") => {
            Some(UiComponentShowcaseDemoEventInput::Hover(true))
        }
        action if action_matches(action, "list_row_pressed") => {
            Some(UiComponentShowcaseDemoEventInput::Press(true))
        }
        action if action_matches(action, "list_row_clicked") => {
            Some(UiComponentShowcaseDemoEventInput::None)
        }
        action if action_matches(action, "virtual_list_scrolled") => {
            Some(UiComponentShowcaseDemoEventInput::SetVisibleRange {
                start: 240,
                count: DEFAULT_VIRTUAL_LIST_VISIBLE_COUNT,
            })
        }
        action if action_matches(action, "paged_list_next_page") => {
            Some(UiComponentShowcaseDemoEventInput::SetPage {
                page_index: 1,
                page_size: DEFAULT_PAGED_LIST_PAGE_SIZE,
            })
        }
        _ => None,
    }
}
