use thiserror::Error;
use zircon_runtime_interface::ui::{tree::UiTreeError, v2::UiV2AssetError};

#[derive(Debug, Error)]
pub(crate) enum ViewTemplateProjectionError {
    #[error("editor view projection requires v2 UI assets, got `{0}`")]
    NonV2AssetPath(String),
    #[error(transparent)]
    V2Asset(#[from] UiV2AssetError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
    #[error("editor view binding rejected `{property}` for control `{control_id}`: {detail}")]
    BindingMutationRejected {
        control_id: String,
        property: String,
        detail: String,
    },
}
