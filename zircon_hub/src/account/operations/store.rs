use super::{
    AccountError, IdentityRevision, JournalFile, OperationJournal, OperationPayload,
    OperationStatus, MAX_JOURNAL_BYTES, MAX_RECORDS,
};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

impl OperationJournal {
    pub(super) fn access<T>(
        &self,
        write: bool,
        action: impl FnOnce(&mut JournalFile) -> Result<T, AccountError>,
    ) -> Result<T, AccountError> {
        self.access_io(write, action)
            .map_err(|_| AccountError::OperationStore)?
    }

    fn access_io<T>(
        &self,
        write: bool,
        action: impl FnOnce(&mut JournalFile) -> Result<T, AccountError>,
    ) -> io::Result<Result<T, AccountError>> {
        if !self.path.is_absolute()
            || self
                .path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(io::Error::other(
                "account journal requires an absolute path",
            ));
        }
        let lock = StoreLock::acquire(self.path.with_extension("lock"))?;
        let mut journal = match lock.open(&self.path, OpenMode::Read) {
            Ok(file) => {
                let mut bytes = Vec::new();
                file.take((MAX_JOURNAL_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > MAX_JOURNAL_BYTES {
                    return Err(io::Error::other("account journal oversized"));
                }
                let bytes = unprotect(&bytes)?;
                let mut journal: JournalFile =
                    serde_json::from_slice(&bytes).map_err(io::Error::other)?;
                if !matches!(journal.version, 1 | 2 | 3 | 4)
                    || journal.operations.len() > MAX_RECORDS
                {
                    return Err(io::Error::other("account journal schema invalid"));
                }
                if journal.operations.iter().any(|record| {
                    (matches!(&record.payload, OperationPayload::CloudCommit { .. })
                        && journal.version < 3)
                        || (matches!(
                            record.status,
                            OperationStatus::Conflict | OperationStatus::OperationIdConflict
                        ) && !matches!(&record.payload, OperationPayload::CloudCommit { .. }))
                        || (record.status == OperationStatus::OperationIdConflict
                            && journal.version < 4)
                }) {
                    return Err(io::Error::other("account journal cloud schema invalid"));
                }
                if journal.version == 1 {
                    let revision = journal.legacy_revision.take().ok_or_else(|| {
                        io::Error::other("account journal legacy revision missing")
                    })?;
                    if !journal.revisions.is_empty() || !canonical_revision(&revision) {
                        return Err(io::Error::other("account journal legacy revision invalid"));
                    }
                    for record in &journal.operations {
                        if !journal
                            .revisions
                            .iter()
                            .any(|entry| entry.identity == record.identity)
                        {
                            journal.revisions.push(IdentityRevision {
                                identity: record.identity.clone(),
                                revision: revision.clone(),
                            });
                        }
                    }
                    journal.version = 2;
                } else if journal.legacy_revision.is_some() {
                    return Err(io::Error::other("account journal revision format invalid"));
                }
                let mut identities = HashSet::with_capacity(journal.revisions.len());
                for entry in &journal.revisions {
                    if !canonical_revision(&entry.revision) || !identities.insert(&entry.identity) {
                        return Err(io::Error::other("account journal revision invalid"));
                    }
                }
                for (index, record) in journal.operations.iter().enumerate() {
                    record
                        .payload
                        .validate(&record.operation_id)
                        .map_err(io::Error::other)?;
                    if record.attempts == 0
                        || !identities.contains(&record.identity)
                        || !record
                            .generation
                            .parse::<u64>()
                            .is_ok_and(|generation| generation.to_string() == record.generation)
                        || journal.operations[..index].iter().any(|other| {
                            other.identity == record.identity
                                && other.operation_id == record.operation_id
                        })
                    {
                        return Err(io::Error::other("account journal identity invalid"));
                    }
                }
                journal
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => JournalFile {
                version: 2,
                revisions: Vec::new(),
                legacy_revision: None,
                operations: Vec::new(),
            },
            Err(error) => return Err(error),
        };
        lock.validate()?;
        let result = action(&mut journal);
        lock.validate()?;
        if write && result.is_ok() {
            self.write(&journal, &lock)?;
        }
        lock.validate()?;
        Ok(result)
    }

    fn write(&self, journal: &JournalFile, lock: &StoreLock) -> io::Result<()> {
        let bytes = serde_json::to_vec(journal).map_err(io::Error::other)?;
        if bytes.len() > MAX_JOURNAL_BYTES {
            return Err(io::Error::other("account journal oversized"));
        }
        let bytes = protect(&bytes)?;
        if bytes.len() > MAX_JOURNAL_BYTES {
            return Err(io::Error::other("protected account journal oversized"));
        }
        let parent = self
            .path
            .parent()
            .ok_or_else(|| io::Error::other("journal parent missing"))?;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let temporary = parent.join(format!(
            ".account-{}-{}-{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(io::Error::other)?
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let result = (|| {
            let mut output = lock.open(&temporary, OpenMode::Create)?;
            output.write_all(&bytes)?;
            output.sync_all()?;
            drop(output);
            lock.validate()?;
            lock.replace(&temporary, &self.path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = lock.remove(&temporary);
        }
        result
    }
}

fn canonical_revision(value: &str) -> bool {
    value
        .parse::<u64>()
        .is_ok_and(|revision| revision.to_string() == value)
}

#[derive(Clone, Copy)]
pub(super) enum OpenMode {
    Read,
    Lock,
    Create,
}

pub(super) fn credential_lock() -> io::Result<StoreLock> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .filter(|base| base.is_absolute())
        .ok_or_else(|| io::Error::other("account credential lock path missing"))?;
    StoreLock::acquire(
        base.join("ZirconHub")
            .join("Account")
            .join("credentials.lock"),
    )
}

// These handles protect per-user runtime storage, never repository source files.
pub(super) struct StoreLock {
    file: File,
    path: PathBuf,
    directories: Vec<(PathBuf, File)>,
}

impl StoreLock {
    fn acquire(path: PathBuf) -> io::Result<Self> {
        if !path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(io::Error::other("account lock path invalid"));
        }
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("account lock parent missing"))?;
        let mut directories: Vec<(PathBuf, File)> = Vec::new();
        for ancestor in parent.ancestors().collect::<Vec<_>>().into_iter().rev() {
            let previous = directories.last().map(|(_, file)| file);
            let directory = open_directory(ancestor, previous)?;
            directories.push((ancestor.to_owned(), directory));
        }
        let directory = &directories
            .last()
            .ok_or_else(|| io::Error::other("account directory missing"))?
            .1;
        // A directory lock also excludes cooperating Unix writers if the lock file is unlinked.
        #[cfg(unix)]
        directory
            .try_lock()
            .map_err(|_| io::Error::other("account directory busy"))?;
        let file = open_in(directory, &path, OpenMode::Lock)?;
        file.try_lock()
            .map_err(|_| io::Error::other("account journal busy"))?;
        let lock = Self {
            file,
            path,
            directories,
        };
        lock.validate()?;
        Ok(lock)
    }

    fn directory(&self) -> &File {
        &self.directories.last().expect("held account directory").1
    }

    fn validate(&self) -> io::Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            for (path, file) in &self.directories {
                let expected = file.metadata()?;
                let current = fs::symlink_metadata(path)?;
                if !current.is_dir()
                    || expected.dev() != current.dev()
                    || expected.ino() != current.ino()
                {
                    return Err(io::Error::other("account directory replaced"));
                }
            }
            let expected = self.file.metadata()?;
            let current = fs::symlink_metadata(&self.path)?;
            if !current.is_file()
                || expected.nlink() != 1
                || expected.dev() != current.dev()
                || expected.ino() != current.ino()
            {
                return Err(io::Error::other("account lock replaced"));
            }
        }
        #[cfg(windows)]
        {
            // Every ancestor and the lock deny DELETE sharing for this guard's lifetime.
            let _ = (&self.file, &self.path);
        }
        Ok(())
    }

    fn open(&self, path: &Path, mode: OpenMode) -> io::Result<File> {
        if path.parent() != self.path.parent() {
            return Err(io::Error::other("account journal parent mismatch"));
        }
        open_in(self.directory(), path, mode)
    }

    fn replace(&self, source: &Path, target: &Path) -> io::Result<()> {
        #[cfg(windows)]
        {
            super::windows::replace(source, target)
        }
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let directory = self.directory().as_raw_fd();
            let source = component_name(source)?;
            let target = component_name(target)?;
            if unsafe { libc::renameat(directory, source.as_ptr(), directory, target.as_ptr()) }
                != 0
            {
                return Err(io::Error::last_os_error());
            }
            self.directory().sync_all()
        }
        #[cfg(not(any(windows, unix)))]
        {
            let _ = (source, target);
            Err(io::Error::other("account journal platform unavailable"))
        }
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            let name = component_name(path)?;
            if unsafe { libc::unlinkat(self.directory().as_raw_fd(), name.as_ptr(), 0) } != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            fs::remove_file(path)
        }
    }
}
#[cfg(windows)]
fn protect(bytes: &[u8]) -> io::Result<Vec<u8>> {
    super::windows::protect(bytes, false)
}
#[cfg(windows)]
fn unprotect(bytes: &[u8]) -> io::Result<Vec<u8>> {
    super::windows::protect(bytes, true)
}
// Non-Windows fixtures can exercise the journal without impersonating a Windows credential store.
#[cfg(all(not(windows), test))]
fn protect(bytes: &[u8]) -> io::Result<Vec<u8>> {
    Ok(bytes.to_vec())
}
#[cfg(all(not(windows), test))]
fn unprotect(bytes: &[u8]) -> io::Result<Vec<u8>> {
    Ok(bytes.to_vec())
}
#[cfg(all(not(windows), not(test)))]
fn protect(_: &[u8]) -> io::Result<Vec<u8>> {
    Err(io::Error::other("account journal requires Windows"))
}
#[cfg(all(not(windows), not(test)))]
fn unprotect(_: &[u8]) -> io::Result<Vec<u8>> {
    Err(io::Error::other("account journal requires Windows"))
}

#[cfg(windows)]
fn open_directory(path: &Path, _: Option<&File>) -> io::Result<File> {
    super::windows::create_directory(path)
}

#[cfg(windows)]
fn open_in(_: &File, path: &Path, mode: OpenMode) -> io::Result<File> {
    super::windows::open_private(path, mode)
}

#[cfg(unix)]
fn component_name(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(
        path.file_name()
            .ok_or_else(|| io::Error::other("account file name missing"))?
            .as_bytes(),
    )
    .map_err(io::Error::other)
}

#[cfg(unix)]
fn open_directory(path: &Path, parent: Option<&File>) -> io::Result<File> {
    use std::os::fd::{AsRawFd, FromRawFd};
    let flags =
        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC;
    let descriptor = if let Some(parent) = parent {
        let name = component_name(path)?;
        let mut descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
        if descriptor < 0 && io::Error::last_os_error().kind() == io::ErrorKind::NotFound {
            if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0
                && io::Error::last_os_error().kind() != io::ErrorKind::AlreadyExists
            {
                return Err(io::Error::last_os_error());
            }
            descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
        }
        descriptor
    } else {
        unsafe { libc::open(c"/".as_ptr(), flags) }
    };
    if descriptor < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_fd(descriptor) })
}

#[cfg(unix)]
fn open_in(parent: &File, path: &Path, mode: OpenMode) -> io::Result<File> {
    use std::os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, PermissionsExt},
    };
    let name = component_name(path)?;
    let flags = libc::O_RDWR
        | libc::O_CLOEXEC
        | libc::O_NOFOLLOW
        | libc::O_NONBLOCK
        | match mode {
            OpenMode::Read => 0,
            OpenMode::Lock => libc::O_CREAT,
            OpenMode::Create => libc::O_CREAT | libc::O_EXCL,
        };
    let descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags, 0o600) };
    if descriptor < 0 {
        return Err(io::Error::last_os_error());
    }
    let file = unsafe { File::from_raw_fd(descriptor) };
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.nlink() != 1 || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err(io::Error::other("account journal file invalid"));
    }
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

#[cfg(not(any(windows, unix)))]
fn open_directory(_: &Path, _: Option<&File>) -> io::Result<File> {
    Err(io::Error::other("account journal platform unavailable"))
}

#[cfg(not(any(windows, unix)))]
fn open_in(_: &File, _: &Path, _: OpenMode) -> io::Result<File> {
    Err(io::Error::other("account journal platform unavailable"))
}
