use crate::ui::template::UiTemplateBuildError;
use thiserror::Error;
use zircon_runtime_interface::ui::template::UiAssetError;
use zircon_runtime_interface::ui::tree::UiTreeError;

#[derive(Debug, Error)]
pub(crate) enum RuntimeUiManagerError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
}
