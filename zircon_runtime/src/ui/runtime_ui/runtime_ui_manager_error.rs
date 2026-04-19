use crate::ui::template::{UiAssetError, UiTemplateBuildError};
use crate::ui::tree::UiTreeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeUiManagerError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
}
