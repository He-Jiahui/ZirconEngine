use std::fmt;

/// Strict decoding failure for the versioned project-session record format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSessionLockRecordDecodeError {
    message: String,
}

impl ProjectSessionLockRecordDecodeError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ProjectSessionLockRecordDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for ProjectSessionLockRecordDecodeError {}
