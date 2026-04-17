use crate::layout::{MainHostPageLayout, TabStackLayout, WorkbenchLayout};

use super::first_tab_stack::first_tab_stack;

pub(super) fn baseline_main_page_tabs(layout: &WorkbenchLayout) -> Option<TabStackLayout> {
    layout.main_pages.iter().find_map(|page| match page {
        MainHostPageLayout::WorkbenchPage {
            document_workspace, ..
        } => first_tab_stack(document_workspace).cloned(),
        MainHostPageLayout::ExclusiveActivityWindowPage { .. } => None,
    })
}
