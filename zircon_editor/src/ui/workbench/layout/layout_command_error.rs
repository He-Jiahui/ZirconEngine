use std::fmt;

use crate::ui::workbench::view::ViewInstanceId;

use super::{ActivityDrawerSlot, ActivityWindowId, MainPageId, WorkspaceTarget};

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
    NonFiniteSplitRatio {
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
    NonFiniteDrawerExtent {
        slot: ActivityDrawerSlot,
    },
    DrawerMissingTab {
        slot: ActivityDrawerSlot,
        instance_id: ViewInstanceId,
    },
    MissingMainPage {
        page_id: MainPageId,
    },
    DuplicateMainPage {
        page_id: MainPageId,
    },
    MissingActivityWindow {
        page_id: MainPageId,
        window_id: ActivityWindowId,
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
            Self::NonFiniteSplitRatio { workspace, path } => write!(
                f,
                "split ratio must be finite at path {path:?} for {workspace:?}"
            ),
            Self::TargetPathIsNotSplitNode { workspace, path } => {
                write!(
                    f,
                    "target path {path:?} for {workspace:?} is not a split node"
                )
            }
            Self::MissingDrawer { slot } => write!(f, "missing drawer {slot:?}"),
            Self::NonFiniteDrawerExtent { slot } => {
                write!(f, "drawer extent must be finite for {slot:?}")
            }
            Self::DrawerMissingTab { slot, instance_id } => write!(
                f,
                "drawer {slot:?} does not contain target tab {}",
                instance_id.0
            ),
            Self::MissingMainPage { page_id } => {
                write!(f, "missing main page {}", page_id.0)
            }
            Self::DuplicateMainPage { page_id } => {
                write!(f, "duplicate main page {}", page_id.0)
            }
            Self::MissingActivityWindow { page_id, window_id } => write!(
                f,
                "main page {} references missing activity window {}",
                page_id.0, window_id.0
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
