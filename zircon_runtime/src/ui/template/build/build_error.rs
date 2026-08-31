use thiserror::Error;

use zircon_runtime_interface::ui::tree::UiTreeError;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiTemplateBuildError {
    #[error(transparent)]
    Tree(#[from] UiTreeError),
    #[error(
        "duplicate control id {control_id:?} at {duplicate_node_path}; first declared at {first_node_path}"
    )]
    DuplicateControlId {
        control_id: String,
        first_node_path: String,
        duplicate_node_path: String,
    },
    #[error("invalid layout contract at {node_path}: {detail}")]
    InvalidLayoutContract { node_path: String, detail: String },
}
