use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformHostBackendRequestError {
    NotAcceptingQuiesce,
    RequestQueueFull,
    RequestQueueClosed,
}

impl fmt::Display for PlatformHostBackendRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            Self::NotAcceptingQuiesce => "backend is not accepting quiesce requests",
            Self::RequestQueueFull => "platform host request queue is full",
            Self::RequestQueueClosed => "platform host request queue is closed",
        };
        formatter.write_str(reason)
    }
}

impl Error for PlatformHostBackendRequestError {}
