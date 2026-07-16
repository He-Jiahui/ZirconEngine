use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum GatewayError {
    #[error("runtime session is no longer available")]
    SessionLost,
    #[error("runtime access requires a serialized gateway operation")]
    RequiresSerializedAccess,
    #[error("runtime gateway capability `{capability}` is unavailable")]
    CapabilityMissing { capability: &'static str },
    #[error("runtime operation failed: {message}")]
    Runtime { message: String },
    #[error("runtime gateway protocol failed: {message}")]
    Protocol { message: String },
}
