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
    #[error("runtime operation handle {} does not exist", handle.raw())]
    UnknownHandle { handle: ZrRuntimeOperationHandle },
    #[error("runtime operation handle {} has not reached a terminal state", handle.raw())]
    NotTerminal { handle: ZrRuntimeOperationHandle },
}
