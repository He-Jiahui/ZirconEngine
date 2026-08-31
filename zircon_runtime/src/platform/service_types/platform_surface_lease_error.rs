use std::error::Error;
use std::fmt;

use crate::core::framework::window::SurfaceLeaseError;
use crate::platform::ApplicationLifecycleServiceError;
use crate::platform::WindowRegistryError;

/// Driver-level failure for a lease operation that crosses native-window and
/// surface-lifetime ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlatformSurfaceLeaseError {
    Lifecycle(ApplicationLifecycleServiceError),
    Registry(WindowRegistryError),
    Lease(SurfaceLeaseError),
}

impl fmt::Display for PlatformSurfaceLeaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lifecycle(error) => error.fmt(formatter),
            Self::Registry(error) => error.fmt(formatter),
            Self::Lease(error) => error.fmt(formatter),
        }
    }
}

impl Error for PlatformSurfaceLeaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lifecycle(error) => Some(error),
            Self::Registry(error) => Some(error),
            Self::Lease(error) => Some(error),
        }
    }
}
