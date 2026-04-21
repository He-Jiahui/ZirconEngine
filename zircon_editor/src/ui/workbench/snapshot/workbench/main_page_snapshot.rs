use crate::ui::workbench::layout::MainPageId;

use super::{DocumentWorkspaceSnapshot, ViewTabSnapshot};

#[derive(Clone, Debug)]
pub enum MainPageSnapshot {
    Workbench {
        id: MainPageId,
        title: String,
        workspace: DocumentWorkspaceSnapshot,
    },
    Exclusive {
        id: MainPageId,
        title: String,
        view: ViewTabSnapshot,
    },
}
