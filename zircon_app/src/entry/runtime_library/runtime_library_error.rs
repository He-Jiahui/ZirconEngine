use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeLibraryErrorKind {
    General,
    ProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeLibraryError {
    kind: RuntimeLibraryErrorKind,
    message: String,
}

impl RuntimeLibraryError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeLibraryErrorKind::General,
            message: message.into(),
        }
    }

    pub(crate) fn protocol_violation(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeLibraryErrorKind::ProtocolViolation,
            message: message.into(),
        }
    }

    pub(crate) const fn kind(&self) -> RuntimeLibraryErrorKind {
        self.kind
    }

    pub(crate) fn with_cleanup_failure(self, cleanup: &RuntimeLibraryError) -> Self {
        Self {
            kind: self.kind,
            message: format!("{}; cleanup also failed: {cleanup}", self.message),
        }
    }
}

impl fmt::Display for RuntimeLibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for RuntimeLibraryError {}

#[cfg(test)]
mod tests {
    use super::{RuntimeLibraryError, RuntimeLibraryErrorKind};

    #[test]
    fn protocol_violations_retain_a_typed_error_kind() {
        let error = RuntimeLibraryError::protocol_violation("foreign output exceeded its budget");

        assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
        assert_eq!(error.to_string(), "foreign output exceeded its budget");
    }
}
