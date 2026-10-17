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
    /// 所需运行时已不可用；默认服务在写入前无法升级 Core 弱句柄时返回此项。
    #[error("configuration runtime is no longer available")]
    RuntimeUnavailable,
    /// flush 目标尚未成功确认，覆盖目标的尝试已无排队或在途重试并报告错误，或 worker 已退出。
    ///
    /// # 载荷
    /// `path` 是服务捕获的实际文件路径的自有文本；`reason` 保存最近错误或 worker 提前退出的说明。
    /// 已接受的内存配置仍由运行时保存，恢复方式见 [`super::ConfigManager::flush`]。
    #[error("configuration persistence failed for {path}: {reason}")]
    Persistence { path: String, reason: String },
    /// 本次等待预算耗尽时目标仍未确认，已请求的后台工作继续保留。
    ///
    /// # 载荷
    /// `path` 是目标文件路径的自有文本；`timeout` 保留调用者传入的等待预算，不是实测耗时。
    /// 重新等待与错误处理见 [`super::ConfigManager::flush`]。
    #[error("configuration flush timed out for {path} after {timeout:?}")]
    FlushTimedOut { path: String, timeout: Duration },
}
