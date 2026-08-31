use std::error::Error;
use std::fmt;

use super::HostCommandBrokerError;

/// Driver-level access failure for the platform-host command authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostCommandBrokerAccessError {
    Uninstalled,
    AlreadyInstalled,
    Broker(HostCommandBrokerError),
}

impl fmt::Display for HostCommandBrokerAccessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uninstalled => {
                formatter.write_str("platform host command broker is not installed")
            }
            Self::AlreadyInstalled => {
                formatter.write_str("platform host command broker is already installed")
            }
            Self::Broker(error) => error.fmt(formatter),
        }
    }
}

impl Error for HostCommandBrokerAccessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Broker(error) => Some(error),
            Self::Uninstalled | Self::AlreadyInstalled => None,
        }
    }
}

impl From<HostCommandBrokerError> for HostCommandBrokerAccessError {
    fn from(error: HostCommandBrokerError) -> Self {
        Self::Broker(error)
    }
}
