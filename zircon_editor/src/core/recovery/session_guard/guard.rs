use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::{
    SessionGuardError, SessionLockDurability, SessionLockInspection, SessionLockRecord,
    SessionOwnershipLease, create_lock, inspect_lock, new_record, read_lock, remove_lock,
    replace_lock, session_lock_path, unix_millis,
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
        let root = project_root.as_ref();
        let path = session_lock_path(root);
        let ownership = SessionOwnershipLease::acquire(root, &path)?;
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
        inspect_lock(&session_lock_path(project_root.as_ref()))
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
        let root = project_root.as_ref();
        let path = session_lock_path(root);
        let ownership = SessionOwnershipLease::acquire(root, &path)?;
        let record = new_record(now)?;
        let current = read_lock(&path)?;
        if current.as_ref() != Some(expected) {
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
        let mut record = self.record.clone();
        record.heartbeat_unix_millis = unix_millis(now)?;
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
