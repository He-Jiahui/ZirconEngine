use std::error::Error;
use std::fmt;

use crate::core::framework::window::SurfaceLeaseError;

use super::super::window_registry::WindowRegistryError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlatformWindowCloseError {
    Registry(WindowRegistryError),
    Lease(SurfaceLeaseError),
}

impl fmt::Display for PlatformWindowCloseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registry(error) => {
                write!(formatter, "window close registry preflight failed: {error}")
            }
            Self::Lease(error) => write!(
                formatter,
                "window close surface-lease preflight failed: {error}"
            ),
        }
    }
}

impl Error for PlatformWindowCloseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registry(error) => Some(error),
            Self::Lease(error) => Some(error),
        }
    }
}
