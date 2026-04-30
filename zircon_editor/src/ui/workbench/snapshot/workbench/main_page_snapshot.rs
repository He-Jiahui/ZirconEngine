use crate::ui::workbench::layout::{ActivityWindowId, MainPageId};

use super::{DocumentWorkspaceSnapshot, ViewTabSnapshot};

#[derive(Clone, Debug)]
pub enum MainPageSnapshot {
    Workbench {
        id: MainPageId,
        title: String,
        activity_window: ActivityWindowId,
        workspace: DocumentWorkspaceSnapshot,
    },
    Exclusive {
        id: MainPageId,
        title: String,
        view: ViewTabSnapshot,
    },
}
