use thiserror::Error;

use super::WorldHandle;

/// 关卡管理调用的同步错误，携带拥有型诊断上下文。
/// 保存请求一旦接纳，后续执行失败通过票据终态报告，参见 `SceneArtifactTicket`。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum LevelManagerError {
    #[error("desktop project file operations are unavailable on this runtime target")]
    DesktopProjectIoUnavailable,
    /// 取得所需 Runtime 上下文失败。
    #[error("level runtime is no longer available")]
    RuntimeUnavailable,
    /// 默认实现无法再分配新关卡句柄。
    #[error("level handle space is exhausted")]
    HandleSpaceExhausted,
    /// 默认关卡创建失败，`reason` 保存创建链错误的文本。
    #[error("default level creation failed: {reason}")]
    CreateFailed { reason: String },
    /// 资产管理服务不可用，`reason` 保存服务解析错误的文本。
    #[error("asset manager is unavailable: {reason}")]
    AssetManagerUnavailable { reason: String },
    /// 资产管理服务尚无活动项目快照。
    #[error("asset manager has no active project generation")]
    ProjectUnavailable,
    /// 活动项目根与请求不符，`active`、`requested` 分别保存两端路径文本。
    #[error("active project root {active} does not match requested root {requested}")]
    ProjectRootMismatch { active: String, requested: String },
    /// 资源定位符解析失败，`uri` 为输入文本，`reason` 为解析错误。
    #[error("invalid level resource locator {uri}: {reason}")]
    InvalidResourceLocator { uri: String, reason: String },
    /// 关卡加载链失败，`uri` 为定位符文本，`reason` 为底层错误。
    #[error("failed to load level resource {uri}: {reason}")]
    LoadFailed { uri: String, reason: String },
    /// 保存调用在返回票据前失败，异步工作错误另由票据报告。
    #[error("failed to save level {handle:?} to {uri}: {reason}")]
    SaveFailed {
        /// 请求保存的句柄键，不延长对应 World 的登记寿命。
        handle: WorldHandle,
        /// 本次保存目标的定位符文本。
        uri: String,
        /// 同步接纳链返回的错误说明。
        reason: String,
    },
}
