use crate::ui::workbench::layout::{MainHostPageLayout, TabStackLayout, WorkbenchLayout};

use super::first_tab_stack::first_tab_stack;

pub(super) fn baseline_main_page_tabs(layout: &WorkbenchLayout) -> Option<TabStackLayout> {
    layout.main_pages.iter().find_map(|page| {
        let MainHostPageLayout::WorkbenchPage { id, .. } = page else {
            return None;
        };
        layout
            .content_workspace_for_page(id)
            .and_then(first_tab_stack)
            .cloned()
    })
}
