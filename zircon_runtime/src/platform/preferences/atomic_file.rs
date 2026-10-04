use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use crate::core::framework::platform::{
    PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};
use crate::core::resource::io::{stage_atomic_write, sync_parent_directory};

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

    fn storage_path(&self, key: &PreferenceKey) -> Arc<Path> {
        let started = Instant::now();
        let mut state = lock(&self.state);
        if let Some(path) = state.path_cache.paths.get(key).cloned() {
            state.diagnostics.path_cache_hits = state.diagnostics.path_cache_hits.saturating_add(1);
            return path;
        }

        let path: Arc<Path> = self
            .root
            .join(STORAGE_DIRECTORY)
            .join(storage_component(key.namespace()))
            .join(format!(
                "{}.{}",
                storage_component(key.key()),
                STORAGE_EXTENSION
            ))
            .into();
        state.diagnostics.path_build_wall = state
            .diagnostics
            .path_build_wall
            .saturating_add(started.elapsed());
        state.diagnostics.path_cache_misses = state.diagnostics.path_cache_misses.saturating_add(1);
        state.diagnostics.path_builds = state.diagnostics.path_builds.saturating_add(1);
        if state.path_cache.paths.len() >= PATH_CACHE_MAX_ENTRIES {
            if let Some(evicted) = state.path_cache.order.pop_front() {
                state.path_cache.paths.remove(evicted.as_ref());
                state.diagnostics.path_cache_evictions =
                    state.diagnostics.path_cache_evictions.saturating_add(1);
            }
        }
        let cache_key = Arc::new(key.clone());
        state
            .path_cache
            .paths
            .insert(Arc::clone(&cache_key), Arc::clone(&path));
        state.path_cache.order.push_back(cache_key);
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
    paths: HashMap<Arc<PreferenceKey>, Arc<Path>>,
    order: VecDeque<Arc<PreferenceKey>>,
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
        match File::open(path.as_ref()) {
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
        let staged = stage_atomic_write(path.as_ref(), value);
        let staged_wall = staged_started.elapsed();
        let mut state = lock(&self.state);
        state.diagnostics.staged_write_wall = state
            .diagnostics
            .staged_write_wall
            .saturating_add(staged_wall);
        drop(state);
        let staged =
            staged.map_err(|error| map_io_error(PreferenceStorageOperation::Write, error))?;

        let commit_started = Instant::now();
        let committed = staged.commit();
        let commit_wall = commit_started.elapsed();
        let mut state = lock(&self.state);
        state.diagnostics.fsync_wall = state.diagnostics.fsync_wall.saturating_add(commit_wall);
        drop(state);
        committed.map_err(|error| map_io_error(PreferenceStorageOperation::Write, error))
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
        match fs::remove_file(path.as_ref()) {
            Ok(()) => {
                let sync_started = Instant::now();
                let result = sync_parent_directory(path.as_ref());
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
#[path = "tests/atomic_file.rs"]
mod tests;
