use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditorLogError {
    #[error("editor log entry capacity must be greater than zero")]
    InvalidEntryCapacity,
    #[error("editor log byte capacity must be greater than zero")]
    InvalidByteCapacity,
    #[error("editor log event queue entry capacity must be greater than zero")]
    InvalidEventQueueEntryCapacity,
    #[error("editor log event queue byte capacity must be greater than zero")]
    InvalidEventQueueByteCapacity,
    #[error(
        "editor log event queue entry capacity {actual} exceeds authoritative store capacity {maximum}"
    )]
    EventQueueEntryCapacityExceedsStore { maximum: usize, actual: usize },
    #[error(
        "editor log event queue byte capacity {actual} exceeds authoritative store byte capacity {maximum}"
    )]
    EventQueueByteCapacityExceedsStore { maximum: usize, actual: usize },
    #[error("editor log rolling-file byte limit must be greater than zero")]
    InvalidRollingFileByteLimit,
    #[error("editor log message must not be empty")]
    EmptyMessage,
    #[error("editor log message is {actual} bytes; maximum is {maximum} bytes")]
    MessageTooLong { maximum: usize, actual: usize },
    #[error("editor log plugin source must not be empty")]
    EmptyPluginSource,
    #[error("editor log jump target must not be empty")]
    EmptyJumpTarget,
    #[error("editor log entry is {actual} bytes; byte capacity is {capacity} bytes")]
    EntryExceedsByteCapacity { capacity: usize, actual: usize },
    #[error("editor log sequence space is exhausted")]
    SequenceExhausted,
    #[error("editor log rolling-file segment space is exhausted")]
    RollingSegmentExhausted,
    #[error("editor log store invariants were violated")]
    StoreInvariantViolation,
    #[error("system clock precedes the Unix epoch")]
    ClockBeforeUnixEpoch,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
