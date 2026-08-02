use std::time::Duration;

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
