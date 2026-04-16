use thiserror::Error;

use crate::UiTreeError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiTemplateBuildError {
    #[error(transparent)]
    Tree(#[from] UiTreeError),
    #[error("invalid layout contract at {node_path}: {detail}")]
    InvalidLayoutContract { node_path: String, detail: String },
}
