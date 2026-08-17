//! Errors raised while consuming runtime-owned output.

use zircon_runtime_interface::{ZrStatus, ZrStatusCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeForeignOutputErrorKind {
    RuntimeCall,
    ProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeForeignOutputError {
    kind: RuntimeForeignOutputErrorKind,
    message: String,
}

impl RuntimeForeignOutputError {
    pub fn runtime_call(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeForeignOutputErrorKind::RuntimeCall,
            message: message.into(),
        }
    }

    pub fn protocol_violation(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeForeignOutputErrorKind::ProtocolViolation,
            message: message.into(),
        }
    }

    pub const fn kind(&self) -> RuntimeForeignOutputErrorKind {
        self.kind
    }

    pub fn with_cleanup_failure(self, cleanup: &impl std::fmt::Display) -> Self {
        Self::protocol_violation(format!("{}; cleanup also failed: {cleanup}", self.message))
    }

    pub(super) fn from_status(status: ZrStatus, operation: &'static str) -> Option<Self> {
        if status.is_ok() {
            return None;
        }
        let diagnostics = unsafe { status.diagnostics.as_slice() };
        let diagnostics = String::from_utf8_lossy(diagnostics);
        let code = match status.status_code() {
            ZrStatusCode::Ok => "ok",
            ZrStatusCode::Error => "error",
            ZrStatusCode::UnsupportedVersion => "unsupported-version",
            ZrStatusCode::InvalidArgument => "invalid-argument",
            ZrStatusCode::NotFound => "not-found",
            ZrStatusCode::CapabilityDenied => "capability-denied",
            ZrStatusCode::BridgeNotEnabled => "bridge-not-enabled",
            ZrStatusCode::Panic => "panic",
        };
        Some(Self::runtime_call(format!(
            "failed to {operation}: {code}: {diagnostics}"
        )))
    }
}

impl std::fmt::Display for RuntimeForeignOutputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RuntimeForeignOutputError {}
