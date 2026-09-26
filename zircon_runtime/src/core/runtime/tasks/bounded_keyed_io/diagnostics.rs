use std::time::Duration;

/// 单条键控 I/O 通道的容量与终结快照，供设置持久化等调用方判断积压和停机进度。
/// queue_entries 覆盖已保留容量的排队、挂起及执行中条目，不能只按队列长度解释。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoundedKeyedIoDiagnostics {
    pub queue_entries: usize,
    pub retained_bytes: usize,
    pub in_flight: usize,
    pub oldest_age: Duration,
    pub submitted: u64,
    pub completed: u64,
    pub failed: u64,
    pub cancelled: u64,
    pub superseded: u64,
    pub coalesced: u64,
    pub worker_wall: Duration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoundedKeyedIoShutdownReport {
    pub complete: bool,
    pub incomplete_entries: usize,
    pub failed: u64,
    pub cancelled: u64,
    pub diagnostics: BoundedKeyedIoDiagnostics,
}
