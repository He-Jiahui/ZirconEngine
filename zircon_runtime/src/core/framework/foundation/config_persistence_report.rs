/// 异步配置写入器的累计观测值；比较 dirty 与 persisted 代际可判断待落盘状态。
///
/// 计数和时延供诊断使用，不能代替 flush 的成功结果作为持久性确认。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConfigPersistenceReport {
    pub dirty_generation: u64,
    pub persisted_generation: u64,
    pub pending_flushes: u64,
    pub peak_pending_flushes: u64,
    pub flush_attempts: u64,
    pub successful_writes: u64,
    pub failed_writes: u64,
    pub serialized_bytes: u64,
    pub flush_p95_ms: f64,
    pub max_flush_ms: f64,
    pub last_error: Option<String>,
}
