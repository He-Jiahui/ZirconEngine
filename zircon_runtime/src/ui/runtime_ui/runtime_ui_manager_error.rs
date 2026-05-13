use thiserror::Error;
use zircon_runtime_interface::ui::tree::UiTreeError;
use zircon_runtime_interface::ui::v2::UiV2AssetError;

#[derive(Debug, Error)]
pub(crate) enum RuntimeUiManagerError {
    #[error(transparent)]
    V2Asset(#[from] UiV2AssetError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
}
