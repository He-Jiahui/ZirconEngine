use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::{CoreError, CoreShutdownError, TaskGraphShutdownReport};

use super::{owner::RetainedOwner, packet::RetainedPacket};

/// Secondary close failure. Native destroy remains a synchronous foreign call.
#[derive(Clone, Debug)]
pub enum ProductCloseError {
    ProjectWatchers,
    Core(CoreShutdownError),
    Runtime(Arc<dyn Error + Send + Sync>),
    RuntimeShared,
    WrongRuntimeThread,
    Busy,
}

impl ProductCloseError {
    pub fn graph_report(&self) -> Option<&TaskGraphShutdownReport> {
        match self {
            Self::Core(error) => error.graph_report(),
            _ => None,
        }
    }
}

impl fmt::Display for ProductCloseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProjectWatchers => f.write_str("project watchers have not stopped"),
            Self::Core(error) => fmt::Display::fmt(error, f),
            Self::Runtime(error) => write!(f, "runtime session destroy failed: {error}"),
            Self::RuntimeShared => f.write_str("runtime session still has external owners"),
            Self::WrongRuntimeThread => f.write_str("runtime cleanup requires its creating thread"),
            Self::Busy => f.write_str("cleanup is busy; original ownership remains retained"),
        }
    }
}

impl Error for ProductCloseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            Self::Runtime(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

/// Original typed failure and an exact capability to retry its acquired resource packet.
/// Formatting and dropping this error never run cleanup or release unfinished ownership.
#[derive(Clone)]
pub struct ProductCompositionFailure {
    primary: Arc<dyn Error + Send + Sync>,
    pub(in crate::entry) owner: Option<Arc<RetainedOwner>>,
}

impl ProductCompositionFailure {
    pub(in crate::entry) fn before_ownership(error: impl Error + Send + Sync + 'static) -> Self {
        Self {
            primary: Arc::new(error),
            owner: None,
        }
    }

    pub(in crate::entry) fn owned(
        primary: Arc<dyn Error + Send + Sync>,
        packet: RetainedPacket,
        deadline: Instant,
    ) -> Self {
        let owner = RetainedOwner::retain(primary.clone(), packet);
        let _ = owner.retry_until(deadline);
        Self {
            primary,
            owner: Some(owner),
        }
    }

    pub(in crate::entry) fn retained(
        primary: Arc<dyn Error + Send + Sync>,
        packet: RetainedPacket,
    ) -> Self {
        let owner = RetainedOwner::retain(primary.clone(), packet);
        Self {
            primary,
            owner: Some(owner),
        }
    }

    pub(in crate::entry) fn from_owner(owner: Arc<RetainedOwner>) -> Self {
        Self {
            primary: owner.primary.clone(),
            owner: Some(owner),
        }
    }

    pub fn primary(&self) -> &(dyn Error + Send + Sync + 'static) {
        self.primary.as_ref()
    }

    pub fn cleanup_pending(&self) -> bool {
        self.owner.as_ref().is_some_and(|owner| owner.pending())
    }

    /// Retries only this exact original packet, using the caller's cooperative Core budget.
    /// A native session additionally requires its creator thread; DLL destroy is not interruptible.
    pub fn retry_cleanup_until(
        &self,
        deadline: Instant,
    ) -> Result<Option<TaskGraphShutdownReport>, ProductCloseError> {
        match &self.owner {
            Some(owner) => owner.retry_until(deadline),
            None => Ok(None),
        }
    }

    /// Nonblocking snapshot of the latest actual secondary error and graph receipt.
    pub fn cleanup_observation(
        &self,
    ) -> Result<(Option<ProductCloseError>, Option<TaskGraphShutdownReport>), ProductCloseError>
    {
        self.owner
            .as_ref()
            .map_or(Ok((None, None)), |owner| owner.observation())
    }
}

impl From<CoreError> for ProductCompositionFailure {
    fn from(error: CoreError) -> Self {
        Self::before_ownership(error)
    }
}

impl fmt::Display for ProductCompositionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.primary)?;
        if let Some(owner) = &self.owner {
            match owner.observation() {
                Ok((Some(secondary), _)) => write!(f, "; cleanup also failed: {secondary}"),
                Ok(_) if owner.pending() => f.write_str("; original ownership remains retained"),
                Ok(_) => Ok(()),
                Err(_) => f.write_str("; cleanup busy; original ownership remains retained"),
            }
        } else {
            Ok(())
        }
    }
}

impl fmt::Debug for ProductCompositionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProductCompositionFailure")
            .field("primary", &self.primary)
            .field("cleanup_pending", &self.cleanup_pending())
            .finish()
    }
}

impl Error for ProductCompositionFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.primary.as_ref())
    }
}
