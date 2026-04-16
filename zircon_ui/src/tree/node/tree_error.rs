use thiserror::Error;

use crate::UiNodeId;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiTreeError {
    #[error("ui tree is missing node {0:?}")]
    MissingNode(UiNodeId),
    #[error("ui tree is missing parent {0:?}")]
    MissingParent(UiNodeId),
    #[error("ui tree already contains node {0:?}")]
    DuplicateNode(UiNodeId),
    #[error("ui tree node {0:?} is not scrollable")]
    NotScrollable(UiNodeId),
}
