use std::time::Duration;

use thiserror::Error;

/// 配置服务弱运行时句柄与异步持久化边界返回的可区分失败。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ConfigManagerError {
    #[error("configuration runtime is no longer available")]
    RuntimeUnavailable,
    #[error("configuration persistence failed for {path}: {reason}")]
    Persistence { path: String, reason: String },
    #[error("configuration flush timed out for {path} after {timeout:?}")]
    FlushTimedOut { path: String, timeout: Duration },
}
