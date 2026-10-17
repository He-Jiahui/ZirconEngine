//! 测试管理器统一资产、路由及批次错误；索引指向第一个失败事件，前序已成功事件保留。

use thiserror::Error;
use zircon_runtime_interface::ui::tree::UiTreeError;
use zircon_runtime_interface::ui::v2::UiV2AssetError;
use zircon_runtime_interface::ui::window::UiRuntimeEventAdapterError;

/// 测试宿主的资产、树与 ABI 适配错误，以及附带原输入位置的批次错误。
///
/// # 批次约束
/// `index` 是输入迭代顺序的零基位置；首错返回且不携带成功前缀的结果向量。
/// 此类型只描述错误，不执行回滚；前序输入已提交的状态仍由管理器持有。
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
