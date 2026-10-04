use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{RuntimeLibraryError, RuntimeSession};
use crate::entry::product_shutdown::retained_owner::{
    pending_owner_count, runtime_owners, RetainedOwner, RetainedPacket,
};

/// Runtime construction diagnostic with its exact original retained packet, when acquired.
/// The shared App owner registry retains unfinished packets after this value is formatted or dropped.
#[derive(Clone)]
pub struct RuntimeSessionCreateFailure {
    error: RuntimeLibraryError,
    owner: Option<Arc<RetainedOwner>>,
}

impl RuntimeSessionCreateFailure {
    pub(crate) fn ensure_available() -> Result<(), Self> {
        let pending = pending_owner_count();
        if pending == 0 {
            return Ok(());
        }
        Err(RuntimeLibraryError::new(format!(
            "runtime admission pending: {pending} original cleanup owner(s) remain"
        ))
        .into())
    }

    pub(crate) fn retained(error: RuntimeLibraryError, session: RuntimeSession) -> Self {
        let owner =
            RetainedOwner::retain(Arc::new(error.clone()), RetainedPacket::runtime(session));
        Self {
            error,
            owner: Some(owner),
        }
    }

    pub(in crate::entry) fn retained_owner(&self) -> Option<Arc<RetainedOwner>> {
        self.owner.clone()
    }

    /// Retries this original Runtime packet. DLL destroy remains synchronous and requires
    /// the session's real creator thread; any attached local Core uses the same deadline.
    pub fn retry_cleanup(&self) -> Result<(), String> {
        self.owner.as_ref().map_or(Ok(()), |owner| {
            owner
                .retry_until(Instant::now() + Duration::from_secs(5))
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
    }

    /// Nonblocking diagnostic snapshot; busy cleanup is an error observation, never success.
    pub fn cleanup_receipt(&self) -> Option<Result<(), String>> {
        let owner = self.owner.as_ref()?;
        Some(match owner.observation() {
            Err(error) => Err(error.to_string()),
            Ok((Some(error), _)) => Err(error.to_string()),
            Ok(_) if owner.pending() => return None,
            Ok(_) => Ok(()),
        })
    }

    pub fn cleanup_pending(&self) -> bool {
        self.owner.as_ref().is_some_and(|owner| owner.pending())
    }

    pub fn cleanup_recovery_context(&self) -> Option<&'static str> {
        self.cleanup_pending().then_some("original ownership remains retained; retry cleanup on the native session's creating thread")
    }

    /// Formatting only; no cleanup lock wait or foreign call.
    pub fn diagnostic_with_recovery(&self) -> String {
        match self.cleanup_recovery_context() {
            Some(context) => format!("{self}; {context}"),
            None => self.to_string(),
        }
    }

    /// Transfers only the current managed Headless runtime-only packet. A combined packet
    /// remains retained intact and cannot lose its local Core or plugin pins through extraction.
    pub(crate) fn take_session(&mut self) -> Option<RuntimeSession> {
        let owner = self.owner.as_ref()?;
        let session = owner.take_runtime_only()?;
        self.owner.take();
        Some(session)
    }
}

impl From<RuntimeLibraryError> for RuntimeSessionCreateFailure {
    fn from(error: RuntimeLibraryError) -> Self {
        Self { error, owner: None }
    }
}
impl fmt::Debug for RuntimeSessionCreateFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeSessionCreateFailure")
            .field("error", &self.error)
            .field("cleanup_pending", &self.cleanup_pending())
            .finish()
    }
}
impl fmt::Display for RuntimeSessionCreateFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.error, f)
    }
}
impl Error for RuntimeSessionCreateFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

/// Retries the current host thread's Runtime-bearing packets through the single shared owner
/// registry. Runtime-only Headless recovery is preserved; Core-only packets use product recovery.
pub fn retry_runtime_startup_cleanup() -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut first_error = None;
    for owner in runtime_owners() {
        if let Err(error) = owner.retry_until(deadline) {
            first_error.get_or_insert_with(|| error.to_string());
        }
    }
    first_error.map_or(Ok(()), Err)
}

#[cfg(test)]
#[path = "tests/runtime_session_create_failure.rs"]
mod tests;
