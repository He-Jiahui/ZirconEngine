use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;

use super::{DocumentNode, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MainHostPageLayout {
    WorkbenchPage {
        id: MainPageId,
        title: String,
        document_workspace: DocumentNode,
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

    pub(crate) fn document_workspace_mut(&mut self) -> Option<&mut DocumentNode> {
        match self {
            Self::WorkbenchPage {
                document_workspace, ..
            } => Some(document_workspace),
            Self::ExclusiveActivityWindowPage { .. } => None,
        }
    }
}
