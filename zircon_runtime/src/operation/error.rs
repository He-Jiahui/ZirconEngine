use thiserror::Error;
use zircon_runtime_interface::ZrRuntimeOperationHandle;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct RuntimeOperationHandlerError {
    message: String,
}

impl RuntimeOperationHandlerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RuntimeOperationServiceError {
    #[error("runtime operation request uses unsupported ABI version {actual}")]
    UnsupportedAbiVersion { actual: u32 },
    #[error("runtime operation id cannot be empty")]
    EmptyOperationId,
    #[error("runtime operation handler `{operation_id}` is already registered")]
    DuplicateHandler { operation_id: String },
    #[error("runtime operation `{operation_id}` is not registered")]
    UnknownOperation { operation_id: String },
    #[error("runtime operation handle space is exhausted")]
    HandleExhausted,
    #[error("runtime operation queue reached its task capacity of {maximum}")]
    TaskCapacityReached { maximum: usize },
    #[error("runtime operation retained data exceeds its byte capacity of {maximum}")]
    RetainedBytesCapacityReached { maximum: usize },
    #[error("runtime operation deadline timer is unavailable")]
    DeadlineTimerUnavailable,
    #[error("runtime operation request is not valid JSON")]
    InvalidRequest,
    #[error("runtime operation payload encoding failed: {message}")]
    PayloadEncoding { message: String },
    #[error("runtime operation handle {} does not exist", handle.raw())]
    UnknownHandle { handle: ZrRuntimeOperationHandle },
    #[error("runtime operation handle {} has not reached a terminal state", handle.raw())]
    NotTerminal { handle: ZrRuntimeOperationHandle },
    #[error("runtime operation handle {} was cancelled", handle.raw())]
    OperationCancelled { handle: ZrRuntimeOperationHandle },
    #[error("runtime operation handle {} expired", handle.raw())]
    OperationExpired { handle: ZrRuntimeOperationHandle },
    #[error("runtime operation handle {} can no longer be cancelled", handle.raw())]
    NotCancellable { handle: ZrRuntimeOperationHandle },
    #[error("runtime operation handle {} was already harvested", handle.raw())]
    AlreadyHarvested { handle: ZrRuntimeOperationHandle },
}
