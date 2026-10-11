use std::time::Duration;

use serde_json::Value;

use super::{ConfigManagerError, ConfigPersistenceReport};

/// 运行时配置服务的共享边界：写入先更新内存，再异步请求持久化。
///
/// 需要磁盘确认的调用方应在写入后调用有超时的 flush；读取返回 None 也可能表示运行时已释放。
///
/// # 职责与使用时机
///
/// 调用者通过此接口共享运行时配置；内存写入与磁盘确认分别由写入和刷新承担。
/// 实现者负责维护两者之间的错误边界，不能把已接收的持久化请求当作落盘回执。
///
/// # 生命周期
///
/// 默认实现借助弱运行时句柄读写配置；运行时释放后，写入返回不可用错误，读取可能返回 None。
/// 持久化报告只是观测快照，不能替代刷新成功。
///
/// # 参考
///
/// 参见 [Self::set_value]、[Self::flush] 和 [ConfigPersistenceReport]。
pub trait ConfigManager: Send + Sync {
    /// 更新共享配置并请求持久化，供不需要立即等待磁盘确认的调用者使用。
    ///
    /// # 参数
    ///
    /// - `key`：运行时配置存储中的键。
    /// - `value`：交给配置存储拥有的 JSON 值；后续读取取得独立值。
    ///
    /// # 返回与错误
    ///
    /// 成功表示内存写入已接受，不证明磁盘已更新。默认实现可能返回 [ConfigManagerError::RuntimeUnavailable]。
    /// 需要磁盘确认时，随后调用 [Self::flush]；同值写入也可请求重试尚未成功的持久化。
    fn set_value(&self, key: &str, value: Value) -> Result<(), ConfigManagerError>;
    /// 读取当前配置值，供无需等待持久化的消费路径使用。
    ///
    /// # 参数
    ///
    /// - `key`：要查询的配置键。
    ///
    /// # 返回与生命周期
    ///
    /// 返回独立拥有的 JSON 值；它不会随之后的写入自动更新。
    /// 默认实现的 None 同时涵盖键不存在与运行时已释放，不能据此单独判断运行时存活。
    ///
    /// # 参考
    ///
    /// 参见 [Self::set_value]。
    fn get_value(&self, key: &str) -> Option<Value>;
    /// 等待本次调用前已请求的配置代际落盘；超时或写入失败应由调用方处理。
    ///
    /// # 参数
    ///
    /// - `timeout`：等待本次目标代际确认的预算；不要求其他线程停止更新配置。
    ///
    /// # 返回与错误
    ///
    /// 默认实现捕获调用时的 dirty 目标，目标已确认时返回成功；后来的写入仍可能需要下一次刷新。
    /// 目标写入失败或 worker 已退出时返回 [ConfigManagerError::Persistence]，等待超时返回
    /// [ConfigManagerError::FlushTimedOut]。
    ///
    /// # 生命周期
    ///
    /// 超时只结束本次等待，不撤销内存更新，也不取消已请求的后台写入。
    ///
    /// # 参考
    ///
    /// 参见 [Self::set_value] 和 [Self::persistence_report]。
    fn flush(&self, timeout: Duration) -> Result<(), ConfigManagerError>;
    /// 读取持久化诊断快照，供状态展示和性能观测使用。
    ///
    /// # 返回与时间边界
    ///
    /// 返回调用时观察到的代际、尝试计数、时延和最近错误；后台继续运行后，快照即可过时。
    /// 默认实现的 pending 为合并后的 0/1 状态，时延取最近完成尝试的滚动窗口。
    ///
    /// # 参考
    ///
    /// 持久性确认使用 [Self::flush]；字段含义见 [ConfigPersistenceReport]。
    fn persistence_report(&self) -> ConfigPersistenceReport;

    /// 判断当前查询是否取得配置值。
    ///
    /// # 参数与返回
    ///
    /// `key` 是配置键；返回值等同于 [Self::get_value] 是否为 Some。
    /// 它继承读取的不可用语义，不单独证明运行时仍然存活。
    fn contains_key(&self, key: &str) -> bool {
        self.get_value(key).is_some()
    }
}
