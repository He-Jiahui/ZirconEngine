use std::time::Duration;

use thiserror::Error;

/// 配置服务弱运行时句柄与异步持久化边界返回的可区分失败。
///
/// # 使用时机
///
/// 调用者区分运行时已不可用、持久化失败和等待超时；它们承担不同的恢复责任。
/// 超时不等于后台工作已取消，持久化失败也不回滚已写入的内存值。
///
/// # 参考
///
/// 参见 [super::ConfigManager::set_value] 和 [super::ConfigManager::flush]。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ConfigManagerError {
    #[error("configuration runtime is no longer available")]
    RuntimeUnavailable,
    #[error("configuration persistence failed for {path}: {reason}")]
    Persistence { path: String, reason: String },
    #[error("configuration flush timed out for {path} after {timeout:?}")]
    FlushTimedOut { path: String, timeout: Duration },
}
