use thiserror::Error;
use zircon_runtime_interface::ui::tree::UiTreeError;
use zircon_runtime_interface::ui::v2::UiV2AssetError;
use zircon_runtime_interface::ui::window::UiRuntimeEventAdapterError;

#[derive(Debug, Error)]
pub(crate) enum RuntimeUiManagerError {
    #[error(transparent)]
    V2Asset(#[from] UiV2AssetError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
    #[error(transparent)]
    RuntimeEventAdapter(#[from] UiRuntimeEventAdapterError),
    #[error("normalized input batch index {index}: {source}")]
    InputBatch {
        index: usize,
        #[source]
        source: UiTreeError,
    },
    #[error("platform input batch index {index}: {source}")]
    PlatformInputBatch {
        index: usize,
        #[source]
        source: UiTreeError,
    },
    #[error("window input pump batch index {index}: {source}")]
    WindowInputPumpBatch {
        index: usize,
        #[source]
        source: UiTreeError,
    },
    #[error("runtime event batch index {index}: {source}")]
    RuntimeEventBatch {
        index: usize,
        #[source]
        source: Box<RuntimeUiManagerError>,
    },
}
