use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum GatewayError {
    #[error("runtime gateway generation exhausted")]
    GenerationExhausted,
    #[error(
        "runtime gateway generation changed from {expected_generation} to {current_generation}"
    )]
    StaleGeneration {
        expected_generation: u64,
        current_generation: u64,
    },
    #[error("runtime session is no longer available")]
    SessionLost,
    #[error("runtime access requires a serialized gateway operation")]
    RequiresSerializedAccess,
    #[error("runtime viewport-surface transition is already in flight for viewport {viewport}")]
    ViewportSurfaceTransitionInFlight { viewport: u64 },
    #[error("borrowed runtime world access cannot be re-entered from its callback")]
    ReentrantBorrowedWorldAccess,
    #[error("runtime gateway capability `{capability}` is unavailable")]
    CapabilityMissing { capability: &'static str },
    #[error("runtime operation failed: {message}")]
    Runtime { message: String },
    #[error("runtime gateway protocol failed: {message}")]
    Protocol { message: String },
}

impl From<zircon_runtime_host::foreign_output::RuntimeForeignOutputError> for GatewayError {
    fn from(error: zircon_runtime_host::foreign_output::RuntimeForeignOutputError) -> Self {
        use zircon_runtime_host::foreign_output::RuntimeForeignOutputErrorKind;

        match error.kind() {
            RuntimeForeignOutputErrorKind::RuntimeCall => Self::Runtime {
                message: error.to_string(),
            },
            RuntimeForeignOutputErrorKind::ProtocolViolation => Self::Protocol {
                message: error.to_string(),
            },
        }
    }
}
