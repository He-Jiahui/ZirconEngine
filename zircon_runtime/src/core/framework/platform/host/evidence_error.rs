use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformHostEvidenceError {
    EmptyBackendVersion,
    BackendVersionTooLong { actual: usize, maximum: usize },
}

impl fmt::Display for PlatformHostEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBackendVersion => formatter.write_str("platform backend version is empty"),
            Self::BackendVersionTooLong { actual, maximum } => write!(
                formatter,
                "platform backend version has {actual} bytes but maximum is {maximum}"
            ),
        }
    }
}

impl Error for PlatformHostEvidenceError {}
