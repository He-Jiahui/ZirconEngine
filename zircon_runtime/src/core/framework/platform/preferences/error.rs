use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PreferenceStorageErrorKind {
    Unavailable,
    Denied,
    CapacityExceeded,
    CorruptBackend,
    TransientIo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PreferenceStorageOperation {
    Read,
    Write,
    Remove,
    Flush,
}

impl PreferenceStorageOperation {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Remove => "remove",
            Self::Flush => "flush",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreferenceStorageError {
    kind: PreferenceStorageErrorKind,
    operation: PreferenceStorageOperation,
    backend: &'static str,
    message: String,
    source: Option<Arc<dyn Error + Send + Sync>>,
}

impl PreferenceStorageError {
    pub fn new(
        kind: PreferenceStorageErrorKind,
        operation: PreferenceStorageOperation,
        backend: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            operation,
            backend,
            message: message.into(),
            source: None,
        }
    }

    pub fn from_source(
        kind: PreferenceStorageErrorKind,
        operation: PreferenceStorageOperation,
        backend: &'static str,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        let message = source.to_string();
        Self {
            kind,
            operation,
            backend,
            message,
            source: Some(Arc::new(source)),
        }
    }

    pub const fn kind(&self) -> PreferenceStorageErrorKind {
        self.kind
    }

    pub const fn operation(&self) -> PreferenceStorageOperation {
        self.operation
    }

    pub const fn backend(&self) -> &'static str {
        self.backend
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PreferenceStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "preference storage {} failed on {} backend: {}",
            self.operation.as_str(),
            self.backend,
            self.message
        )
    }
}

impl Error for PreferenceStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}
