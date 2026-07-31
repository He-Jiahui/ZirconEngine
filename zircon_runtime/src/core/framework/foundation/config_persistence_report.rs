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
