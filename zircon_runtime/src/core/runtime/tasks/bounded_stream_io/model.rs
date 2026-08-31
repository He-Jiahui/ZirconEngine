use std::fmt;
use std::io::Read;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::super::TaskGraphAdmissionError;

pub const DEFAULT_BOUNDED_STREAM_IO_MAX_CONCURRENT_READERS: usize = 16;
pub const DEFAULT_BOUNDED_STREAM_IO_READ_CHUNK_BYTES: usize = 8 * 1024;
pub const DEFAULT_BOUNDED_STREAM_IO_MAX_LINE_BYTES: usize = 64 * 1024;
pub const DEFAULT_BOUNDED_STREAM_IO_QUEUE_ENTRY_CAPACITY: usize = 1_024;
pub const DEFAULT_BOUNDED_STREAM_IO_QUEUE_BYTE_CAPACITY: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundedStreamIoLimits {
    pub max_concurrent_readers: usize,
    pub read_chunk_bytes: usize,
    pub max_line_bytes: usize,
    pub queue_entry_capacity: usize,
    pub queue_byte_capacity: usize,
}

impl BoundedStreamIoLimits {
    pub const fn with_max_concurrent_readers(mut self, value: usize) -> Self {
        self.max_concurrent_readers = value;
        self
    }

    pub const fn with_read_chunk_bytes(mut self, value: usize) -> Self {
        self.read_chunk_bytes = value;
        self
    }

    pub const fn with_max_line_bytes(mut self, value: usize) -> Self {
        self.max_line_bytes = value;
        self
    }

    pub const fn with_queue_entry_capacity(mut self, value: usize) -> Self {
        self.queue_entry_capacity = value;
        self
    }

    pub const fn with_queue_byte_capacity(mut self, value: usize) -> Self {
        self.queue_byte_capacity = value;
        self
    }

    pub(crate) fn validate(self) -> Result<Self, BoundedStreamIoLimitError> {
        if self.max_concurrent_readers == 0 {
            return Err(BoundedStreamIoLimitError::ZeroConcurrentReaders);
        }
        if self.read_chunk_bytes == 0 {
            return Err(BoundedStreamIoLimitError::ZeroReadChunkBytes);
        }
        if self.max_line_bytes == 0 {
            return Err(BoundedStreamIoLimitError::ZeroLineBytes);
        }
        if self.queue_entry_capacity == 0 {
            return Err(BoundedStreamIoLimitError::ZeroQueueEntries);
        }
        if self.queue_byte_capacity == 0 {
            return Err(BoundedStreamIoLimitError::ZeroQueueBytes);
        }
        Ok(self)
    }
}

impl Default for BoundedStreamIoLimits {
    fn default() -> Self {
        Self {
            max_concurrent_readers: DEFAULT_BOUNDED_STREAM_IO_MAX_CONCURRENT_READERS,
            read_chunk_bytes: DEFAULT_BOUNDED_STREAM_IO_READ_CHUNK_BYTES,
            max_line_bytes: DEFAULT_BOUNDED_STREAM_IO_MAX_LINE_BYTES,
            queue_entry_capacity: DEFAULT_BOUNDED_STREAM_IO_QUEUE_ENTRY_CAPACITY,
            queue_byte_capacity: DEFAULT_BOUNDED_STREAM_IO_QUEUE_BYTE_CAPACITY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundedStreamIoLimitError {
    ZeroConcurrentReaders,
    ZeroReadChunkBytes,
    ZeroLineBytes,
    ZeroQueueEntries,
    ZeroQueueBytes,
}

impl fmt::Display for BoundedStreamIoLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ZeroConcurrentReaders => "bounded stream I/O reader capacity must be non-zero",
            Self::ZeroReadChunkBytes => "bounded stream I/O read chunk must be non-zero",
            Self::ZeroLineBytes => "bounded stream I/O line limit must be non-zero",
            Self::ZeroQueueEntries => "bounded stream I/O queue entry capacity must be non-zero",
            Self::ZeroQueueBytes => "bounded stream I/O queue byte capacity must be non-zero",
        })
    }
}

impl std::error::Error for BoundedStreamIoLimitError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundedStreamIoDrainBudget {
    pub max_records: usize,
    pub max_bytes: usize,
    pub max_time: Duration,
}

impl BoundedStreamIoDrainBudget {
    pub const fn new(max_records: usize, max_bytes: usize, max_time: Duration) -> Self {
        Self {
            max_records,
            max_bytes,
            max_time,
        }
    }

    pub const fn unlimited() -> Self {
        Self::new(usize::MAX, usize::MAX, Duration::MAX)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoundedStreamIoStreamId(Arc<str>);

impl BoundedStreamIoStreamId {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    pub fn stdout() -> Self {
        Self::new("stdout")
    }

    pub fn stderr() -> Self {
        Self::new("stderr")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for BoundedStreamIoStreamId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub struct BoundedStreamIoReader {
    pub(crate) stream: BoundedStreamIoStreamId,
    pub(crate) reader: Box<dyn Read + Send>,
}

impl BoundedStreamIoReader {
    pub fn new(stream: BoundedStreamIoStreamId, reader: impl Read + Send + 'static) -> Self {
        Self {
            stream,
            reader: Box::new(reader),
        }
    }
}

impl fmt::Debug for BoundedStreamIoReader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BoundedStreamIoReader")
            .field("stream", &self.stream)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct BoundedStreamIoRecord {
    pub stream: BoundedStreamIoStreamId,
    pub text: String,
    pub truncated_bytes: u64,
    pub lossy_utf8: bool,
    pub captured_at: Instant,
    pub(crate) retained_bytes: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoundedStreamIoDiagnostics {
    pub queued_records: usize,
    pub queued_bytes: usize,
    pub peak_queued_records: usize,
    pub peak_queued_bytes: usize,
    pub dropped_records: u64,
    pub dropped_bytes: u64,
    pub truncated_records: u64,
    pub truncated_bytes: u64,
    pub lossy_utf8_records: u64,
    pub completed_readers: u64,
    pub cancelled_readers: u64,
    pub failed_readers: u64,
    pub active_readers: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoundedStreamIoLaneDiagnostics {
    pub active_readers: usize,
    pub peak_active_readers: usize,
    pub admitted_readers: u64,
    pub rejected_readers: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundedStreamIoFailure {
    pub stream: BoundedStreamIoStreamId,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct BoundedStreamIoBatch {
    pub records: Vec<BoundedStreamIoRecord>,
    pub drained_bytes: usize,
    pub oldest_age: Duration,
    pub elapsed: Duration,
    pub diagnostics: BoundedStreamIoDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundedStreamIoAdmissionError {
    InvalidLimits(BoundedStreamIoLimitError),
    EmptyOwner,
    EmptyReaders,
    EmptyStreamId,
    ReaderCapacityReached { requested: usize, available: usize },
    TaskHandleSpaceExhausted,
    Execution(TaskGraphAdmissionError),
}

impl fmt::Display for BoundedStreamIoAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits(error) => error.fmt(formatter),
            Self::EmptyOwner => formatter.write_str("bounded stream I/O owner must not be empty"),
            Self::EmptyReaders => {
                formatter.write_str("bounded stream I/O capture requires at least one reader")
            }
            Self::EmptyStreamId => {
                formatter.write_str("bounded stream I/O stream identity must not be empty")
            }
            Self::ReaderCapacityReached {
                requested,
                available,
            } => write!(
                formatter,
                "bounded stream I/O requested {requested} readers but only {available} are available"
            ),
            Self::TaskHandleSpaceExhausted => {
                formatter.write_str("bounded stream I/O task handle space is exhausted")
            }
            Self::Execution(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for BoundedStreamIoAdmissionError {}

impl From<BoundedStreamIoLimitError> for BoundedStreamIoAdmissionError {
    fn from(error: BoundedStreamIoLimitError) -> Self {
        Self::InvalidLimits(error)
    }
}

impl From<TaskGraphAdmissionError> for BoundedStreamIoAdmissionError {
    fn from(error: TaskGraphAdmissionError) -> Self {
        Self::Execution(error)
    }
}
