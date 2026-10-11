/// 异步配置写入器的累计观测值；比较 dirty 与 persisted 代际可判断待落盘状态。
///
/// 计数和时延供诊断使用，不能代替 flush 的成功结果作为持久性确认。
///
/// # 使用时机与字段
///
/// 此值用于展示、比较和诊断持久化进展。dirty 与 persisted 表示请求与成功确认边界；
/// 尝试计数包含成功和失败，序列化字节数也包含随后写入失败的尝试。
///
/// # 时间边界
///
/// 默认实现逐次完成尝试后更新计数；p95 与最大耗时使用最近最多 64 次完成尝试，
/// 并非整个运行期的历史最大值。快照不阻止后续请求和写入。
///
/// # 参考
///
/// 参见 [super::ConfigManager::persistence_report]；需要磁盘确认时使用 [super::ConfigManager::flush]。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConfigPersistenceReport {
    pub dirty_generation: u64,
    pub persisted_generation: u64,
    /// 默认实现以 0/1 表示是否有待落盘代际；peak_pending_flushes 记录这个状态的峰值。
    pub pending_flushes: u64,
    pub peak_pending_flushes: u64,
    /// 后台已完成持久化尝试的累计次数，成功和失败均计入。
    pub flush_attempts: u64,
    pub successful_writes: u64,
    pub failed_writes: u64,
    /// 各次尝试成功生成的 JSON 字节数之和；写入失败时也累计该次字节数。
    pub serialized_bytes: u64,
    /// 默认实现取最近最多 64 次完成尝试的 p95，单位毫秒，包含失败尝试。
    /// 耗时包含快照读取、序列化和写入。
    pub flush_p95_ms: f64,
    /// 默认实现最近最多 64 次完成尝试的最大耗时（毫秒），包含失败尝试。
    pub max_flush_ms: f64,
    pub last_error: Option<String>,
}
