use thiserror::Error;
use zircon_ui::template::{UiAssetError, UiTemplateBuildError};
use zircon_ui::tree::UiTreeError;

#[derive(Debug, Error)]
pub enum RuntimeUiManagerError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
}
