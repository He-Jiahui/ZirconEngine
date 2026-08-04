use std::collections::{HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::core::framework::platform::{
    PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};
use crate::core::resource::io::atomic_file::stage_atomic_write;

use super::{
    PreferenceBackendWorkAuthority, PreferenceStorageBackend, PreferenceStorageBackendDiagnostics,
};

const BACKEND_NAME: &str = "atomic_file";
const STORAGE_DIRECTORY: &str = "preferences-v1";
const STORAGE_EXTENSION: &str = "zrpref";
const PATH_CACHE_MAX_ENTRIES: usize = 4096;

#[derive(Clone, Debug)]
pub struct AtomicFilePreferenceStorageBackend {
    root: PathBuf,
    state: Arc<Mutex<AtomicFilePreferenceStorageState>>,
}

impl AtomicFilePreferenceStorageBackend {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            state: Arc::new(Mutex::new(AtomicFilePreferenceStorageState::default())),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn storage_path(&self, key: &PreferenceKey) -> PathBuf {
        let started = Instant::now();
        let mut state = lock(&self.state);
        if let Some(path) = state.path_cache.paths.get(key).cloned() {
            state.diagnostics.path_cache_hits = state.diagnostics.path_cache_hits.saturating_add(1);
            return path;
        }

        let path = self
            .root
            .join(STORAGE_DIRECTORY)
            .join(storage_component(key.namespace()))
            .join(format!(
                "{}.{}",
                storage_component(key.key()),
                STORAGE_EXTENSION
            ));
        state.diagnostics.path_build_wall = state
            .diagnostics
            .path_build_wall
            .saturating_add(started.elapsed());
        state.diagnostics.path_cache_misses = state.diagnostics.path_cache_misses.saturating_add(1);
        state.diagnostics.path_builds = state.diagnostics.path_builds.saturating_add(1);
        if state.path_cache.paths.len() >= PATH_CACHE_MAX_ENTRIES {
            if let Some(evicted) = state.path_cache.order.pop_front() {
                state.path_cache.paths.remove(&evicted);
                state.diagnostics.path_cache_evictions =
                    state.diagnostics.path_cache_evictions.saturating_add(1);
            }
        }
        state.path_cache.paths.insert(key.clone(), path.clone());
        state.path_cache.order.push_back(key.clone());
        state.diagnostics.path_cache_entries = state.path_cache.paths.len() as u64;
        path
    }
}

#[derive(Debug, Default)]
struct AtomicFilePreferenceStorageState {
    diagnostics: PreferenceStorageBackendDiagnostics,
    path_cache: StoragePathCache,
}

#[derive(Debug, Default)]
struct StoragePathCache {
    paths: HashMap<PreferenceKey, PathBuf>,
    order: VecDeque<PreferenceKey>,
}

impl PreferenceStorageBackend for AtomicFilePreferenceStorageBackend {
    fn backend_kind(&self) -> PreferenceStorageBackendKind {
        PreferenceStorageBackendKind::AtomicFile
    }

    fn open_read(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<Option<Box<dyn io::Read + Send>>, PreferenceStorageError> {
        let path = self.storage_path(key);
        let mut state = lock(&self.state);
        state.diagnostics.reads = state.diagnostics.reads.saturating_add(1);
        drop(state);
        match File::open(path) {
            Ok(value) => Ok(Some(Box::new(value))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(map_io_error(PreferenceStorageOperation::Read, error)),
        }
    }

    fn write(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
        value: &[u8],
    ) -> Result<(), PreferenceStorageError> {
        let path = self.storage_path(key);
        let mut state = lock(&self.state);
        state.diagnostics.writes = state.diagnostics.writes.saturating_add(1);
        drop(state);

        let staged_started = Instant::now();
        let staged = stage_atomic_write(&path, value);
        let mut state = lock(&self.state);
        state.diagnostics.staged_write_wall = state
            .diagnostics
            .staged_write_wall
            .saturating_add(staged_started.elapsed());
        drop(state);
        let staged =
            staged.map_err(|error| map_io_error(PreferenceStorageOperation::Write, error))?;

        let commit_started = Instant::now();
        let committed = staged.commit();
        let mut state = lock(&self.state);
        state.diagnostics.staged_write_wall = state
            .diagnostics
            .staged_write_wall
            .saturating_add(commit_started.elapsed());
        drop(state);
        committed.map_err(|error| map_io_error(PreferenceStorageOperation::Write, error))?;

        let sync_started = Instant::now();
        let synced = sync_committed_value(&path);
        let mut state = lock(&self.state);
        state.diagnostics.fsync_wall = state
            .diagnostics
            .fsync_wall
            .saturating_add(sync_started.elapsed());
        drop(state);
        synced.map_err(|error| map_io_error(PreferenceStorageOperation::Write, error))
    }

    fn remove(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<(), PreferenceStorageError> {
        let path = self.storage_path(key);
        let mut state = lock(&self.state);
        state.diagnostics.removes = state.diagnostics.removes.saturating_add(1);
        drop(state);
        match fs::remove_file(&path) {
            Ok(()) => {
                let sync_started = Instant::now();
                let result = sync_parent_directory(&path);
                let mut state = lock(&self.state);
                state.diagnostics.fsync_wall = state
                    .diagnostics
                    .fsync_wall
                    .saturating_add(sync_started.elapsed());
                drop(state);
                result.map_err(|error| map_io_error(PreferenceStorageOperation::Remove, error))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(map_io_error(PreferenceStorageOperation::Remove, error)),
        }
    }

    fn flush(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), PreferenceStorageError> {
        let mut state = lock(&self.state);
        state.diagnostics.flushes = state.diagnostics.flushes.saturating_add(1);
        // Each atomic write synchronizes its committed value before returning.
        Ok(())
    }

    fn diagnostics(&self) -> PreferenceStorageBackendDiagnostics {
        lock(&self.state).diagnostics
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn storage_component(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

fn sync_committed_value(path: &Path) -> io::Result<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?
        .sync_all()?;
    sync_parent_directory(path)
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    // Windows uses the shared ReplaceFileW commit path after the staged file is synced.
    Ok(())
}

fn map_io_error(operation: PreferenceStorageOperation, error: io::Error) -> PreferenceStorageError {
    let kind = match error.kind() {
        io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => {
            PreferenceStorageErrorKind::Denied
        }
        io::ErrorKind::StorageFull | io::ErrorKind::FileTooLarge | io::ErrorKind::QuotaExceeded => {
            PreferenceStorageErrorKind::CapacityExceeded
        }
        io::ErrorKind::InvalidData
        | io::ErrorKind::NotADirectory
        | io::ErrorKind::IsADirectory
        | io::ErrorKind::AlreadyExists => PreferenceStorageErrorKind::CorruptBackend,
        _ => PreferenceStorageErrorKind::TransientIo,
    };
    PreferenceStorageError::from_source(kind, operation, BACKEND_NAME, error)
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use crate::core::framework::platform::{
        PreferenceStorageErrorKind, PreferenceStorageOperation,
    };

    use super::{map_io_error, AtomicFilePreferenceStorageBackend, PATH_CACHE_MAX_ENTRIES};
    use crate::core::framework::platform::{PreferenceKey, PreferenceStorageBackend};

    #[test]
    fn platform_preference_storage_path_cache_is_stable_and_bounded() {
        let backend = AtomicFilePreferenceStorageBackend::new("cache-test-root");
        let key = PreferenceKey::new("woc.input", "bindings").unwrap();

        let first = backend.storage_path(&key);
        let second = backend.storage_path(&key);
        assert_eq!(first, second);

        for index in 0..PATH_CACHE_MAX_ENTRIES {
            let key = PreferenceKey::new("woc.cache", format!("entry-{index}")).unwrap();
            let _ = backend.storage_path(&key);
        }

        let diagnostics = backend.diagnostics();
        assert_eq!(diagnostics.path_cache_hits, 1);
        assert_eq!(
            diagnostics.path_cache_misses,
            PATH_CACHE_MAX_ENTRIES as u64 + 1
        );
        assert_eq!(diagnostics.path_builds, diagnostics.path_cache_misses);
        assert_eq!(
            diagnostics.path_cache_entries,
            PATH_CACHE_MAX_ENTRIES as u64
        );
        assert_eq!(diagnostics.path_cache_evictions, 1);

        let _ = backend.storage_path(&key);
        let diagnostics = backend.diagnostics();
        assert_eq!(diagnostics.path_cache_hits, 1);
        assert_eq!(
            diagnostics.path_cache_misses,
            PATH_CACHE_MAX_ENTRIES as u64 + 2
        );
        assert_eq!(diagnostics.path_cache_evictions, 2);
    }

    #[test]
    fn platform_preference_storage_maps_host_io_error_categories() {
        let cases = [
            (
                io::ErrorKind::PermissionDenied,
                PreferenceStorageErrorKind::Denied,
            ),
            (
                io::ErrorKind::StorageFull,
                PreferenceStorageErrorKind::CapacityExceeded,
            ),
            (
                io::ErrorKind::FileTooLarge,
                PreferenceStorageErrorKind::CapacityExceeded,
            ),
            (
                io::ErrorKind::QuotaExceeded,
                PreferenceStorageErrorKind::CapacityExceeded,
            ),
            (
                io::ErrorKind::ReadOnlyFilesystem,
                PreferenceStorageErrorKind::Denied,
            ),
            (
                io::ErrorKind::InvalidData,
                PreferenceStorageErrorKind::CorruptBackend,
            ),
            (
                io::ErrorKind::Other,
                PreferenceStorageErrorKind::TransientIo,
            ),
        ];

        for (host_kind, expected) in cases {
            let error = map_io_error(
                PreferenceStorageOperation::Write,
                io::Error::new(host_kind, "injected preference storage failure"),
            );
            assert_eq!(error.kind(), expected);
            assert_eq!(error.operation(), PreferenceStorageOperation::Write);
            assert_eq!(error.backend(), "atomic_file");
            assert!(error.source().is_some());
        }
    }
}
