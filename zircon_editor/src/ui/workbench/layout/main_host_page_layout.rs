use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;

use super::{ActivityWindowId, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MainHostPageLayout {
    WorkbenchPage {
        id: MainPageId,
        title: String,
        activity_window: ActivityWindowId,
    },
    ExclusiveActivityWindowPage {
        id: MainPageId,
        title: String,
        window_instance: ViewInstanceId,
    },
}

impl MainHostPageLayout {
    pub fn id(&self) -> &MainPageId {
        match self {
            Self::WorkbenchPage { id, .. } | Self::ExclusiveActivityWindowPage { id, .. } => id,
        }
    }

    pub fn activity_window_id(&self) -> Option<&ActivityWindowId> {
        match self {
            Self::WorkbenchPage {
                activity_window, ..
            } => Some(activity_window),
            Self::ExclusiveActivityWindowPage { .. } => None,
        }
    }
}
