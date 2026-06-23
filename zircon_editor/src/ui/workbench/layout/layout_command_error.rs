use std::fmt;

use crate::ui::workbench::view::ViewInstanceId;

use super::{ActivityDrawerSlot, MainPageId, WorkspaceTarget};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutCommandError {
    MissingWorkspacePath {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
    },
    MissingSplitPath {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
    },
    TargetPathIsNotSplitNode {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
    },
    MissingDrawer {
        slot: ActivityDrawerSlot,
    },
    DrawerMissingTab {
        slot: ActivityDrawerSlot,
        instance_id: ViewInstanceId,
    },
    MissingDocumentNode {
        page_id: MainPageId,
        path: Vec<usize>,
    },
    DocumentSplitAttachTarget {
        page_id: MainPageId,
        path: Vec<usize>,
    },
    MissingFloatingWindow {
        window_id: MainPageId,
    },
    MissingFloatingWindowNode {
        window_id: MainPageId,
        path: Vec<usize>,
    },
    FloatingSplitAttachTarget {
        window_id: MainPageId,
        path: Vec<usize>,
    },
}

impl fmt::Display for LayoutCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWorkspacePath { workspace, path } => {
                write!(f, "missing workspace path {path:?} for {workspace:?}")
            }
            Self::MissingSplitPath { workspace, path } => {
                write!(f, "missing split path {path:?} for {workspace:?}")
            }
            Self::TargetPathIsNotSplitNode { workspace, path } => {
                write!(
                    f,
                    "target path {path:?} for {workspace:?} is not a split node"
                )
            }
            Self::MissingDrawer { slot } => write!(f, "missing drawer {slot:?}"),
            Self::DrawerMissingTab { slot, instance_id } => write!(
                f,
                "drawer {slot:?} does not contain target tab {}",
                instance_id.0
            ),
            Self::MissingDocumentNode { page_id, path } => {
                write!(f, "missing document node {path:?} on page {}", page_id.0)
            }
            Self::DocumentSplitAttachTarget { page_id, path } => write!(
                f,
                "cannot attach directly to split node {path:?} on page {}",
                page_id.0
            ),
            Self::MissingFloatingWindow { window_id } => {
                write!(f, "missing floating window {}", window_id.0)
            }
            Self::MissingFloatingWindowNode { window_id, path } => write!(
                f,
                "missing floating window node {path:?} on window {}",
                window_id.0
            ),
            Self::FloatingSplitAttachTarget { window_id, path } => write!(
                f,
                "cannot attach directly to split node {path:?} on floating window {}",
                window_id.0
            ),
        }
    }
}

impl std::error::Error for LayoutCommandError {}
