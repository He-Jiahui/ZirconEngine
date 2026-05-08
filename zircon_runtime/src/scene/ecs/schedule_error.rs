use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ScheduleError {
    #[error("system id cannot be empty")]
    EmptySystemId,
    #[error("system {0} already registered")]
    DuplicateSystem(String),
}
