use thiserror::Error;
use zircon_ui::{UiAssetError, UiTemplateBuildError, UiTreeError};

#[derive(Debug, Error)]
pub enum RuntimeUiManagerError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
}
