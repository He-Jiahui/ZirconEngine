use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditorRuntimeEventConsumerError {
    #[error("runtime event consumer `{consumer_id}` is already registered")]
    DuplicateConsumer { consumer_id: String },
    #[error("runtime event consumer host already owns play session {play_session_id}")]
    SessionAlreadyActive { play_session_id: u64 },
    #[error("runtime event consumer host has no active play session")]
    NoActiveSession,
    #[error("runtime gateway session {actual} does not match play session {expected}")]
    RuntimeSessionMismatch { expected: u64, actual: u64 },
    #[error("runtime does not expose plugin event subscriptions for `{consumer_id}`")]
    Unsupported { consumer_id: String },
    #[error("runtime event consumer `{consumer_id}` gateway call failed: {message}")]
    Gateway {
        consumer_id: String,
        message: String,
    },
    #[error(
        "runtime event consumer `{consumer_id}` received session {actual}, expected {expected}"
    )]
    WrongSession {
        consumer_id: String,
        expected: u64,
        actual: u64,
    },
    #[error("runtime event consumer `{consumer_id}` received a foreign subscription")]
    ForeignSubscription { consumer_id: String },
    #[error(
        "runtime event consumer `{consumer_id}` expected event `{expected}`, received `{actual}`"
    )]
    EventMismatch {
        consumer_id: String,
        expected: String,
        actual: String,
    },
    #[error(
        "runtime event consumer `{consumer_id}` expected schema `{expected}`, received `{actual}`"
    )]
    SchemaMismatch {
        consumer_id: String,
        expected: String,
        actual: String,
    },
    #[error("runtime event consumer `{consumer_id}` rejected stale sequence {sequence}")]
    StaleSequence { consumer_id: String, sequence: u64 },
    #[error("runtime event consumer `{consumer_id}` rejected its payload: {source}")]
    Payload {
        consumer_id: String,
        #[source]
        source: EditorRuntimeEventConsumerApplyError,
    },
    #[error("{operation} failed: {primary}; cleanup also failed: {cleanup}")]
    Cleanup {
        operation: &'static str,
        #[source]
        primary: Box<EditorRuntimeEventConsumerError>,
        cleanup: Box<EditorRuntimeEventConsumerError>,
    },
}

impl EditorRuntimeEventConsumerError {
    pub(super) fn with_cleanup(operation: &'static str, primary: Self, cleanup: Self) -> Self {
        Self::Cleanup {
            operation,
            primary: Box::new(primary),
            cleanup: Box::new(cleanup),
        }
    }
}

#[derive(Debug, Error)]
pub enum EditorRuntimeEventConsumerApplyError {
    #[error("payload decoding failed: {source}")]
    Decode {
        #[source]
        source: serde_json::Error,
    },
    #[error("consumer state rejected the payload: {source}")]
    State {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl EditorRuntimeEventConsumerApplyError {
    pub fn state(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::State {
            source: Box::new(source),
        }
    }
}
