use crate::ui::workbench::layout::{ActivityWindowId, MainPageId};
use crate::ui::workbench::view::ActivityWindowTemplateSpec;

use super::{DocumentWorkspaceSnapshot, ViewTabSnapshot};

#[derive(Clone, Debug)]
pub enum MainPageSnapshot {
    Workbench {
        id: MainPageId,
        title: String,
        activity_window: ActivityWindowId,
        activity_window_template: Option<ActivityWindowTemplateSpec>,
        workspace: DocumentWorkspaceSnapshot,
    },
    Exclusive {
        id: MainPageId,
        title: String,
        view: ViewTabSnapshot,
    },
}
