use std::time::Duration;

use serde_json::Value;

use super::{ConfigManagerError, ConfigPersistenceReport};

/// 运行时配置服务的共享边界：写入先更新内存，再异步请求持久化。
///
/// 需要磁盘确认的调用方应在写入后调用有超时的 flush；读取返回 None 也可能表示运行时已释放。
pub trait ConfigManager: Send + Sync {
    fn set_value(&self, key: &str, value: Value) -> Result<(), ConfigManagerError>;
    fn get_value(&self, key: &str) -> Option<Value>;
    /// 等待本次调用前已请求的配置代际落盘；超时或写入失败应由调用方处理。
    fn flush(&self, timeout: Duration) -> Result<(), ConfigManagerError>;
    fn persistence_report(&self) -> ConfigPersistenceReport;

    fn contains_key(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }
}
