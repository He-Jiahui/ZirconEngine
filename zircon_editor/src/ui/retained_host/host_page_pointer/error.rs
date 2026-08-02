use thiserror::Error;
use zircon_runtime_interface::ui::tree::UiTreeError;

#[derive(Debug, Error)]
pub(crate) enum HostPagePointerError {
    #[error("invalid measured host page tab frame for index {item_index}: x={x}, width={width}")]
    InvalidTabFrame {
        item_index: usize,
        x: f32,
        width: f32,
    },
    #[error("failed to dispatch host page pointer event: {0}")]
    Dispatch(#[from] UiTreeError),
}
