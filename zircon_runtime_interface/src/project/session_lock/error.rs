use std::fmt;

/// Strict validation failure for the versioned project-session admission record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSessionAdmissionRecordError {
    message: String,
}

impl ProjectSessionAdmissionRecordError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ProjectSessionAdmissionRecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for ProjectSessionAdmissionRecordError {}
