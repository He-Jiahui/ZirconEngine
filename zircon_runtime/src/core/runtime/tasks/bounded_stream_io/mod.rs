//! Runtime-owned bounded decoding and lifecycle accounting for blocking streams.

mod capture;
mod decoder;
mod lane;
mod model;
mod state;
mod worker;

pub use capture::BoundedStreamIoCapture;
pub use lane::BoundedStreamIoLane;
pub use model::{
    BoundedStreamIoAdmissionError, BoundedStreamIoBatch, BoundedStreamIoDiagnostics,
    BoundedStreamIoDrainBudget, BoundedStreamIoFailure, BoundedStreamIoLaneDiagnostics,
    BoundedStreamIoLimitError, BoundedStreamIoLimits, BoundedStreamIoReader, BoundedStreamIoRecord,
    BoundedStreamIoStreamId, DEFAULT_BOUNDED_STREAM_IO_MAX_CONCURRENT_READERS,
    DEFAULT_BOUNDED_STREAM_IO_MAX_LINE_BYTES, DEFAULT_BOUNDED_STREAM_IO_QUEUE_BYTE_CAPACITY,
    DEFAULT_BOUNDED_STREAM_IO_QUEUE_ENTRY_CAPACITY, DEFAULT_BOUNDED_STREAM_IO_READ_CHUNK_BYTES,
};

#[cfg(test)]
mod tests;
