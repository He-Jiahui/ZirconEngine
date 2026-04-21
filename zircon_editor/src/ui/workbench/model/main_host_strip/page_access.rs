use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::MainPageSnapshot;

use super::active_view::active_view_in_workspace;

pub(super) fn page_id(page: &MainPageSnapshot) -> &MainPageId {
    match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => id,
    }
}

pub(super) fn page_title(page: &MainPageSnapshot) -> &str {
    match page {
        MainPageSnapshot::Workbench { title, .. } | MainPageSnapshot::Exclusive { title, .. } => {
            title
        }
    }
}

pub(super) fn page_dirty(page: &MainPageSnapshot) -> bool {
    match page {
        MainPageSnapshot::Workbench { workspace, .. } => active_view_in_workspace(workspace)
            .map(|view| view.dirty)
            .unwrap_or(false),
        MainPageSnapshot::Exclusive { view, .. } => view.dirty,
    }
}
