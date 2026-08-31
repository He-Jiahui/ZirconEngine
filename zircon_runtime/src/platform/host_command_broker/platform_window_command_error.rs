use std::error::Error;
use std::fmt;

use crate::platform::{
    HostCommandBrokerAccessError, WindowRegistryError, WindowStateRegistryError,
};

/// The driver-level result for an atomic window command admission transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlatformWindowCommandError {
    Registry(WindowRegistryError),
    State(WindowStateRegistryError),
    Broker(HostCommandBrokerAccessError),
}

impl fmt::Display for PlatformWindowCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(error) => error.fmt(formatter),
            Self::State(error) => error.fmt(formatter),
            Self::Broker(error) => error.fmt(formatter),
        }
    }
}

impl Error for PlatformWindowCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registry(error) => Some(error),
            Self::State(error) => Some(error),
            Self::Broker(error) => Some(error),
        }
    }
}
