use std::error::Error;
use std::fmt;

use crate::platform::{ApplicationLifecycleServiceError, PlatformSurfaceLeaseError};

/// Failure to begin or complete the lifecycle-owned surface suspend protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlatformApplicationSuspendError {
    Lifecycle(ApplicationLifecycleServiceError),
    Surface(PlatformSurfaceLeaseError),
    SurfaceLeasesPending {
        active_count: usize,
        preparing_count: usize,
        retiring_count: usize,
    },
}

impl fmt::Display for PlatformApplicationSuspendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lifecycle(error) => error.fmt(formatter),
            Self::Surface(error) => error.fmt(formatter),
            Self::SurfaceLeasesPending {
                active_count,
                preparing_count,
                retiring_count,
            } => write!(
                formatter,
                "cannot finish application suspension with {active_count} active, {preparing_count} preparing, and {retiring_count} retiring surface leases"
            ),
        }
    }
}

impl Error for PlatformApplicationSuspendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Lifecycle(error) => Some(error),
            Self::Surface(error) => Some(error),
            Self::SurfaceLeasesPending { .. } => None,
        }
    }
}
