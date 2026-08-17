use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RuntimeEventMirrorError {
    #[error("runtime event mirror id cannot be empty")]
    EmptyEventId,
    #[error("runtime event mirror `{event_id}` payload schema cannot be empty")]
    EmptyPayloadSchema { event_id: String },
    #[error(
        "runtime event mirror `{event_id}` {field} is {actual_bytes} bytes, maximum is {max_bytes}"
    )]
    DescriptorTooLarge {
        event_id: String,
        field: &'static str,
        actual_bytes: usize,
        max_bytes: usize,
    },
    #[error("runtime event mirror `{event_id}` is already registered")]
    DuplicateEventId { event_id: String },
    #[error("runtime event mirror `{event_id}` is not registered")]
    UnknownEventId { event_id: String },
    #[error(
        "runtime event mirror `{event_id}` expects payload schema `{expected}`, received `{actual}`"
    )]
    PayloadSchemaMismatch {
        event_id: String,
        expected: String,
        actual: String,
    },
    #[error("runtime event mirror `{event_id}` could not connect its ECS reader")]
    ConnectionFailed { event_id: String },
    #[error("runtime event mirror `{event_id}` reader count overflowed")]
    ReaderCountOverflow { event_id: String },
    #[error("runtime event mirror `{event_id}` reader count underflowed")]
    ReaderCountUnderflow { event_id: String },
    #[error("runtime event mirror `{event_id}` subscription is disconnected")]
    Disconnected { event_id: String },
    #[error("runtime event mirror `{event_id}` failed to serialize its payload: {message}")]
    Serialize { event_id: String, message: String },
    #[error(
        "runtime event mirror `{event_id}` payload is {payload_bytes} bytes, page maximum is {max_payload_bytes}"
    )]
    PayloadTooLarge {
        event_id: String,
        payload_bytes: usize,
        max_payload_bytes: usize,
    },
    #[error(
        "runtime event mirror `{event_id}` payload nesting depth is {observed_depth}, maximum is {max_depth}"
    )]
    PayloadTooDeep {
        event_id: String,
        observed_depth: usize,
        max_depth: usize,
    },
    #[error(
        "runtime event mirror `{event_id}` payload processing exceeded {limit_micros} microseconds"
    )]
    ProcessingTime { event_id: String, limit_micros: u64 },
    #[error(
        "runtime event mirror `{event_id}` queue overflowed at {pending_events}/{max_events} events and {pending_payload_bytes}/{max_payload_bytes} payload bytes"
    )]
    QueueOverflow {
        event_id: String,
        pending_events: usize,
        pending_payload_bytes: usize,
        max_events: usize,
        max_payload_bytes: usize,
    },
    #[error("runtime event mirror `{event_id}` reader-count callback failed: {message}")]
    ReaderCountCallback { event_id: String, message: String },
}
