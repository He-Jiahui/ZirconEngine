use std::path::{Path, PathBuf};
use std::time::SystemTime;

use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime_interface::project::session_lock::ProjectSessionAdmissionLifecycleV1;

use super::{
    ProjectSessionAdmissionRecordV1, SessionAdmissionRequest, SessionGuardError,
    SessionLockDurability, SessionLockInspection, SessionOwnershipLease, create_lock, inspect_lock,
    new_record, next_session_generation, read_lock, remove_lock, replace_lock, session_lock_path,
    unix_millis,
};

/// Owns one project session lock until normal shutdown explicitly releases it.
#[derive(Debug)]
pub struct SessionGuard {
    path: PathBuf,
    record: ProjectSessionAdmissionRecordV1,
    durability: SessionLockDurability,
    ownership: Option<SessionOwnershipLease>,
    released: bool,
}

impl SessionGuard {
    /// Claims the project admission boundary without implicitly replacing a residual lock.
    pub fn claim(
        project_root: impl AsRef<Path>,
        admission: &SessionAdmissionRequest,
    ) -> Result<super::SessionGuardAdmission, SessionGuardError> {
        Self::claim_at(project_root, admission, SystemTime::now())
    }

    pub fn claim_at(
        project_root: impl AsRef<Path>,
        admission: &SessionAdmissionRequest,
        now: SystemTime,
    ) -> Result<super::SessionGuardAdmission, SessionGuardError> {
        super::liveness::claim(project_root, admission, now)
    }

    pub(super) fn create_with_owned_lease(
        path: PathBuf,
        ownership: SessionOwnershipLease,
        admission: &SessionAdmissionRequest,
        now: SystemTime,
    ) -> Result<Self, SessionGuardError> {
        let record = new_record(admission, now)?;
        let durability = create_lock(&path, &record)?;
        Ok(Self {
            path,
            record,
            durability,
            ownership: Some(ownership),
            released: false,
        })
    }

    pub fn inspect(
        project_root: impl AsRef<Path>,
    ) -> Result<SessionLockInspection, SessionGuardError> {
        let root = resolve_project_root(project_root.as_ref())?;
        inspect_lock(&session_lock_path(&root))
    }

    /// Replaces exactly the residual record selected by recovery policy.
    ///
    /// The caller must present the record it inspected after the project lifecycle owner has
    /// established that no live editor still owns the project. The guard retains an OS lease
    /// from this verification through release, so a concurrent session operation cannot replace
    /// the selected record between this check and the new record publication.
    pub fn replace_residual_at(
        project_root: impl AsRef<Path>,
        expected: &ProjectSessionAdmissionRecordV1,
        admission: &SessionAdmissionRequest,
        now: SystemTime,
    ) -> Result<Self, SessionGuardError> {
        let root = resolve_project_root(project_root.as_ref())?;
        let path = session_lock_path(&root);
        let ownership = SessionOwnershipLease::acquire(&root, &path)?;
        Self::replace_with_owned_lease(path, expected.clone(), ownership, admission, now)
    }

    pub(super) fn replace_with_owned_lease(
        path: PathBuf,
        expected: ProjectSessionAdmissionRecordV1,
        ownership: SessionOwnershipLease,
        admission: &SessionAdmissionRequest,
        now: SystemTime,
    ) -> Result<Self, SessionGuardError> {
        let record = new_record(admission, now)?;
        let current = read_lock(&path)?;
        if current.as_ref() != Some(&expected) {
            return Err(SessionGuardError::OwnershipLost { path });
        }
        let durability = replace_lock(&path, &record)?;
        Ok(Self {
            path,
            record,
            durability,
            ownership: Some(ownership),
            released: false,
        })
    }

    pub fn record(&self) -> &ProjectSessionAdmissionRecordV1 {
        &self.record
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the canonical root whose `.zircon/session.lock` this guard exclusively owns.
    pub fn project_root(&self) -> &Path {
        self.path
            .parent()
            .and_then(Path::parent)
            .expect("SessionGuard lock path always resides below <project>/.zircon")
    }

    pub const fn is_released(&self) -> bool {
        self.released
    }

    pub const fn durability(&self) -> SessionLockDurability {
        self.durability
    }

    pub fn refresh_heartbeat(&mut self) -> Result<SessionLockDurability, SessionGuardError> {
        self.refresh_heartbeat_at(SystemTime::now())
    }

    pub fn refresh_heartbeat_at(
        &mut self,
        now: SystemTime,
    ) -> Result<SessionLockDurability, SessionGuardError> {
        self.ensure_owned()?;
        let record = self.record.with_heartbeat_unix_millis(unix_millis(now)?);
        let durability = replace_lock(&self.path, &record)?;
        self.record = record;
        self.durability = durability;
        Ok(durability)
    }

    /// Records that the data-only project receipt was accepted under this lease.
    pub fn mark_preflight_approved(&mut self) -> Result<SessionLockDurability, SessionGuardError> {
        self.transition_to(ProjectSessionAdmissionLifecycleV1::PreflightApproved)
    }

    /// Records the start of effectful project activation after approved preflight.
    pub fn begin_activation(&mut self) -> Result<SessionLockDurability, SessionGuardError> {
        self.transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
    }

    /// Commits the generation that Hub and focus consumers may subsequently address.
    pub fn commit_ready(&mut self) -> Result<SessionLockDurability, SessionGuardError> {
        self.ensure_owned()?;
        let record = self
            .record
            .commit_ready(next_session_generation()?)
            .map_err(|error| self.invalid_lifecycle(error))?;
        self.replace_record(record)
    }

    /// Retains the OS lease while a failed activation still needs recovery or operator action.
    pub fn mark_recovery_required(&mut self) -> Result<SessionLockDurability, SessionGuardError> {
        if self.record.lifecycle() == ProjectSessionAdmissionLifecycleV1::RecoveryRequired {
            return Ok(self.durability);
        }
        self.transition_to(ProjectSessionAdmissionLifecycleV1::RecoveryRequired)
    }

    /// Stops Ready-only consumers before teardown starts while retaining exclusive ownership.
    pub fn begin_close(&mut self) -> Result<SessionLockDurability, SessionGuardError> {
        if self.record.lifecycle() == ProjectSessionAdmissionLifecycleV1::Closing {
            return Ok(self.durability);
        }
        self.transition_to(ProjectSessionAdmissionLifecycleV1::Closing)
    }

    /// Persists a residual recovery marker and relinquishes only the live ownership lease.
    ///
    /// Unlike normal `release`, this deliberately retains `session.lock`. A later process must
    /// observe it as a residual record and pass the explicit recovery takeover policy before it
    /// can establish a new writer session.
    pub fn release_ownership_for_recovery(
        &mut self,
    ) -> Result<SessionLockDurability, SessionGuardError> {
        if self.released {
            return if self.record.lifecycle()
                == ProjectSessionAdmissionLifecycleV1::RecoveryRequired
            {
                Ok(self.durability)
            } else {
                Err(SessionGuardError::OwnershipLost {
                    path: self.path.clone(),
                })
            };
        }
        self.ensure_owned()?;
        self.mark_recovery_required()?;
        self.released = true;
        self.ownership.take();
        Ok(self.durability)
    }

    /// Normal shutdown must call this explicitly so an I/O failure cannot be silently discarded.
    pub fn release(&mut self) -> Result<SessionLockDurability, SessionGuardError> {
        if self.released {
            return Ok(self.durability);
        }
        self.ensure_owned()?;
        if !matches!(
            self.record.lifecycle(),
            ProjectSessionAdmissionLifecycleV1::Closing
                | ProjectSessionAdmissionLifecycleV1::RecoveryRequired
        ) {
            self.begin_close()?;
        }
        let durability = remove_lock(&self.path)?;
        self.released = true;
        self.durability = durability;
        self.ownership.take();
        Ok(durability)
    }

    fn ensure_owned(&self) -> Result<(), SessionGuardError> {
        if self.released
            || self.ownership.is_none()
            || read_lock(&self.path)?.as_ref() != Some(&self.record)
        {
            return Err(SessionGuardError::OwnershipLost {
                path: self.path.clone(),
            });
        }
        Ok(())
    }

    fn transition_to(
        &mut self,
        lifecycle: ProjectSessionAdmissionLifecycleV1,
    ) -> Result<SessionLockDurability, SessionGuardError> {
        self.ensure_owned()?;
        let record = self
            .record
            .transition_to(lifecycle)
            .map_err(|error| self.invalid_lifecycle(error))?;
        self.replace_record(record)
    }

    fn replace_record(
        &mut self,
        record: ProjectSessionAdmissionRecordV1,
    ) -> Result<SessionLockDurability, SessionGuardError> {
        let durability = replace_lock(&self.path, &record)?;
        self.record = record;
        self.durability = durability;
        Ok(durability)
    }

    fn invalid_lifecycle(
        &self,
        error: zircon_runtime_interface::project::session_lock::ProjectSessionAdmissionRecordError,
    ) -> SessionGuardError {
        SessionGuardError::InvalidRecord {
            path: self.path.clone(),
            message: error.to_string(),
        }
    }
}

fn resolve_project_root(project_root: &Path) -> Result<PathBuf, SessionGuardError> {
    ProjectPaths::resolve_path(project_root)
        .map(|root| root.into_operation_path())
        .map_err(|source| SessionGuardError::Io {
            operation: "resolve project session path",
            path: project_root.to_path_buf(),
            source,
        })
}
