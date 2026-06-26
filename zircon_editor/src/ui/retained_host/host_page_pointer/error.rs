use thiserror::Error;
use zircon_runtime_interface::ui::tree::UiTreeError;

#[derive(Debug, Error)]
pub(crate) enum HostPagePointerError {
    #[error("failed to dispatch host page pointer event: {0}")]
    Dispatch(#[from] UiTreeError),
}
