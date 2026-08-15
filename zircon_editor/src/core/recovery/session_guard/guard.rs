use std::path::{Path, PathBuf};
use std::time::SystemTime;

use zircon_runtime::asset::project::ProjectPaths;

use super::{
    create_lock, inspect_lock, new_record, read_lock, remove_lock, replace_lock, session_lock_path,
    unix_millis, SessionGuardError, SessionLockDurability, SessionLockInspection,
    SessionLockRecord, SessionOwnershipLease,
};

/// Owns one project session lock until normal shutdown explicitly releases it.
#[derive(Debug)]
pub struct SessionGuard {
    path: PathBuf,
    record: SessionLockRecord,
    durability: SessionLockDurability,
    ownership: Option<SessionOwnershipLease>,
    released: bool,
}

impl SessionGuard {
    pub fn acquire(project_root: impl AsRef<Path>) -> Result<Self, SessionGuardError> {
        Self::acquire_at(project_root, SystemTime::now())
    }

    pub fn acquire_at(
        project_root: impl AsRef<Path>,
        now: SystemTime,
    ) -> Result<Self, SessionGuardError> {
        let root = resolve_project_root(project_root.as_ref())?;
        let path = session_lock_path(&root);
        let ownership = SessionOwnershipLease::acquire(&root, &path)?;
        Self::create_with_owned_lease(path, ownership, now)
    }

    /// Claims the project admission boundary without implicitly replacing a residual lock.
    pub fn claim(
        project_root: impl AsRef<Path>,
    ) -> Result<super::SessionGuardAdmission, SessionGuardError> {
        super::liveness::claim(project_root)
    }

    pub(super) fn create_with_owned_lease(
        path: PathBuf,
        ownership: SessionOwnershipLease,
        now: SystemTime,
    ) -> Result<Self, SessionGuardError> {
        let record = new_record(now)?;
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
        expected: &SessionLockRecord,
        now: SystemTime,
    ) -> Result<Self, SessionGuardError> {
        let root = resolve_project_root(project_root.as_ref())?;
        let path = session_lock_path(&root);
        let ownership = SessionOwnershipLease::acquire(&root, &path)?;
        Self::replace_with_owned_lease(path, expected.clone(), ownership, now)
    }

    pub(super) fn replace_with_owned_lease(
        path: PathBuf,
        expected: SessionLockRecord,
        ownership: SessionOwnershipLease,
        now: SystemTime,
    ) -> Result<Self, SessionGuardError> {
        let record = new_record(now)?;
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

    pub fn record(&self) -> &SessionLockRecord {
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

    /// Normal shutdown must call this explicitly so an I/O failure cannot be silently discarded.
    pub fn release(&mut self) -> Result<SessionLockDurability, SessionGuardError> {
        if self.released {
            return Ok(self.durability);
        }
        self.ensure_owned()?;
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
