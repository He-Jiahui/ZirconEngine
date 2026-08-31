use thiserror::Error;

/// Rejects the reserved nil launch-process identity.
#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum ProjectLaunchInstanceIdError {
    #[error("project launch instance id must not be nil")]
    Nil,
}
