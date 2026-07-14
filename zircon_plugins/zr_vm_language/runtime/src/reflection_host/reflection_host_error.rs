use std::fmt;

use zircon_runtime::script::VmReflectionError;

use crate::CallSiteError;

/// Typed failures returned by the ZrVM numeric reflection host module.
#[derive(Debug)]
pub enum ReflectionHostError {
    /// No VM schema has been installed for this package instance.
    Uninitialized,
    /// Dense call-site compilation or execution failed.
    CallSite(CallSiteError),
    /// The shared VM reflection schema projection failed.
    Schema(VmReflectionError),
    /// No active scene runtime context was available for a field operation.
    RuntimeContext(String),
    /// A reflected value JSON payload could not be encoded or decoded.
    Json(serde_json::Error),
}

impl fmt::Display for ReflectionHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uninitialized => {
                formatter.write_str("VM reflection host schema is not initialized")
            }
            Self::CallSite(error) => error.fmt(formatter),
            Self::Schema(error) => error.fmt(formatter),
            Self::RuntimeContext(message) => formatter.write_str(message),
            Self::Json(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ReflectionHostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CallSite(error) => Some(error),
            Self::Schema(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Uninitialized | Self::RuntimeContext(_) => None,
        }
    }
}

impl From<CallSiteError> for ReflectionHostError {
    fn from(error: CallSiteError) -> Self {
        Self::CallSite(error)
    }
}

impl From<VmReflectionError> for ReflectionHostError {
    fn from(error: VmReflectionError) -> Self {
        Self::Schema(error)
    }
}

impl From<serde_json::Error> for ReflectionHostError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
