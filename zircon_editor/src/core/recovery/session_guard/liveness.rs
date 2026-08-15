use std::path::{Path, PathBuf};
use std::time::SystemTime;

use zircon_runtime::asset::project::ProjectPaths;

use super::{
    read_lock, session_lock_path, SessionGuard, SessionGuardError, SessionLockRecord,
    SessionOwnershipLease,
};

/// The result of atomically claiming a project's session admission boundary.
///
/// An active owner retains the OS lease, while a residual record is held by this value until
/// recovery explicitly takes it over or drops the claim. This keeps process liveness distinct
/// from persisted crash-recovery data.
#[derive(Debug)]
pub enum SessionGuardAdmission {
    Acquired(SessionGuard),
    Active { record: Option<SessionLockRecord> },
    Residual(SessionGuardResidual),
}

/// A residual record protected by the ownership lease until recovery makes an explicit choice.
#[derive(Debug)]
pub struct SessionGuardResidual {
    path: PathBuf,
    record: SessionLockRecord,
    ownership: Option<SessionOwnershipLease>,
}

impl SessionGuardResidual {
    pub fn record(&self) -> &SessionLockRecord {
        &self.record
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Replaces this exact residual record while retaining the lease through publication.
    pub fn take_over_at(mut self, now: SystemTime) -> Result<SessionGuard, SessionGuardError> {
        let path = self.path.clone();
        let ownership = self
            .ownership
            .take()
            .ok_or_else(|| SessionGuardError::OwnershipLost { path })?;
        SessionGuard::replace_with_owned_lease(self.path, self.record, ownership, now)
    }

    pub fn take_over(self) -> Result<SessionGuard, SessionGuardError> {
        self.take_over_at(SystemTime::now())
    }
}

pub(super) fn claim(
    project_root: impl AsRef<Path>,
) -> Result<SessionGuardAdmission, SessionGuardError> {
    let root = ProjectPaths::resolve_path(project_root.as_ref())
        .map(|root| root.into_operation_path())
        .map_err(|source| SessionGuardError::Io {
            operation: "resolve project session path",
            path: project_root.as_ref().to_path_buf(),
            source,
        })?;
    let path = session_lock_path(&root);
    let ownership = match SessionOwnershipLease::acquire(&root, &path) {
        Ok(ownership) => ownership,
        Err(SessionGuardError::AlreadyHeld { record, .. }) => {
            return Ok(SessionGuardAdmission::Active { record });
        }
        Err(error) => return Err(error),
    };

    match read_lock(&path)? {
        Some(record) => Ok(SessionGuardAdmission::Residual(SessionGuardResidual {
            path,
            record,
            ownership: Some(ownership),
        })),
        None => SessionGuard::create_with_owned_lease(path, ownership, SystemTime::now())
            .map(SessionGuardAdmission::Acquired),
    }
}
