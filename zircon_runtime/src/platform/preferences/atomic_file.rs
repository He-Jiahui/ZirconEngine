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
mod tests {
    use std::collections::{HashMap, VecDeque};
    use std::error::Error;
    use std::hint::black_box;
    use std::io;
    use std::sync::Arc;
    use std::time::Instant;

    use crate::core::framework::platform::{
        PreferenceStorageErrorKind, PreferenceStorageOperation,
    };

    use super::{
        map_io_error, AtomicFilePreferenceStorageBackend, PreferenceStorageBackend,
        PATH_CACHE_MAX_ENTRIES,
    };
    use crate::core::framework::platform::PreferenceKey;

    #[test]
    fn platform_preference_storage_path_cache_is_stable_and_bounded() {
        let backend = AtomicFilePreferenceStorageBackend::new("cache-test-root");
        let key = PreferenceKey::new("woc.input", "bindings").unwrap();

        let first = backend.storage_path(&key);
        let second = backend.storage_path(&key);
        assert_eq!(first, second);
        assert!(Arc::ptr_eq(&first, &second));
        {
            let state = backend.state.lock().unwrap();
            let map_key = state.path_cache.paths.keys().next().unwrap();
            let fifo_key = state.path_cache.order.front().unwrap();
            assert!(Arc::ptr_eq(map_key, fifo_key));
        }

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
    #[ignore = "managed release performance evidence"]
    fn platform_preference_storage_path_cache_shared_clone_release_benchmark_evidence() {
        for (case, root_repeats, target_p95_ratio_pct) in
            [("short", 8, 75), ("medium", 64, 50), ("long", 320, 50)]
        {
            benchmark_shared_path_clone_case(case, root_repeats, target_p95_ratio_pct);
        }
        benchmark_shared_key_retention();
    }

    fn benchmark_shared_key_retention() {
        const ENTRIES_PER_SAMPLE: usize = PATH_CACHE_MAX_ENTRIES;
        const SAMPLE_PAIRS: usize = 21;
        const TARGET_P95_RATIO_PCT: u128 = 75;

        let namespace = "n".repeat(128);
        let key_suffix = "k".repeat(506);
        let keys = (0..ENTRIES_PER_SAMPLE)
            .map(|index| {
                PreferenceKey::new(namespace.as_str(), format!("{index:06}{key_suffix}"))
                    .expect("maximum-size benchmark key must remain valid")
            })
            .collect::<Vec<_>>();
        let key_bytes_per_entry = keys[0].namespace().len() + keys[0].key().len();
        let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut shared_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);

        for sample_index in 0..SAMPLE_PAIRS {
            let mut measure_legacy = || {
                legacy_samples_ns.push(measure_legacy_key_retention(&keys));
            };
            let mut measure_shared = || {
                shared_samples_ns.push(measure_shared_key_retention(&keys));
            };
            if sample_index % 2 == 0 {
                measure_legacy();
                measure_shared();
            } else {
                measure_shared();
                measure_legacy();
            }
        }

        let legacy_p50_ns = nearest_rank_percentile(&legacy_samples_ns, 50);
        let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
        let shared_p50_ns = nearest_rank_percentile(&shared_samples_ns, 50);
        let shared_p95_ns = nearest_rank_percentile(&shared_samples_ns, 95);
        let legacy_ns = join_samples(&legacy_samples_ns);
        let shared_ns = join_samples(&shared_samples_ns);
        let legacy_key_clones = ENTRIES_PER_SAMPLE * 2 * SAMPLE_PAIRS;
        let shared_key_clones = ENTRIES_PER_SAMPLE * SAMPLE_PAIRS;
        let legacy_string_clones = legacy_key_clones * 2;
        let shared_string_clones = shared_key_clones * 2;
        let legacy_copied_key_bytes = key_bytes_per_entry * legacy_key_clones;
        let shared_copied_key_bytes = key_bytes_per_entry * shared_key_clones;

        println!(
            "PREFERENCE_PATH_CACHE_KEY_BENCH_V1 entries_per_sample={ENTRIES_PER_SAMPLE} \
             key_bytes_per_entry={key_bytes_per_entry} sample_pairs={SAMPLE_PAIRS} \
             legacy_key_clones={legacy_key_clones} shared_key_clones={shared_key_clones} \
             legacy_string_clones={legacy_string_clones} shared_string_clones={shared_string_clones} \
             legacy_copied_key_bytes={legacy_copied_key_bytes} \
             shared_copied_key_bytes={shared_copied_key_bytes} \
             legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             shared_p50_ns={shared_p50_ns} shared_p95_ns={shared_p95_ns} \
             target_p95_ratio_pct={TARGET_P95_RATIO_PCT} legacy_ns={legacy_ns} shared_ns={shared_ns}"
        );
        assert!(
            shared_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_RATIO_PCT),
            "shared key-retention P95 {shared_p95_ns}ns must be at most {TARGET_P95_RATIO_PCT}% of legacy P95 {legacy_p95_ns}ns"
        );
    }

    fn measure_legacy_key_retention(keys: &[PreferenceKey]) -> u128 {
        let started = Instant::now();
        let mut paths = HashMap::with_capacity(keys.len());
        let mut order = VecDeque::with_capacity(keys.len());
        for (index, key) in keys.iter().enumerate() {
            paths.insert(key.clone(), index);
            order.push_back(key.clone());
        }
        black_box((&paths, &order));
        started.elapsed().as_nanos()
    }

    fn measure_shared_key_retention(keys: &[PreferenceKey]) -> u128 {
        let started = Instant::now();
        let mut paths = HashMap::with_capacity(keys.len());
        let mut order = VecDeque::with_capacity(keys.len());
        for (index, key) in keys.iter().enumerate() {
            let shared_key = Arc::new(key.clone());
            paths.insert(Arc::clone(&shared_key), index);
            order.push_back(shared_key);
        }
        black_box((&paths, &order));
        started.elapsed().as_nanos()
    }

    fn benchmark_shared_path_clone_case(
        case: &str,
        root_repeats: usize,
        target_p95_ratio_pct: u128,
    ) {
        const CLONES_PER_SAMPLE: usize = 4_096;
        const SAMPLE_PAIRS: usize = 21;

        let root = "cache-segment\\".repeat(root_repeats);
        let backend = AtomicFilePreferenceStorageBackend::new(root);
        let key = PreferenceKey::new("woc.input", "bindings").unwrap();
        let shared_path = backend.storage_path(&key);
        let legacy_path = shared_path.as_ref().to_path_buf();
        let path_bytes = legacy_path.as_os_str().len();
        let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
        let mut shared_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);

        for sample_index in 0..SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                for _ in 0..CLONES_PER_SAMPLE {
                    black_box(legacy_path.clone());
                }
                legacy_samples_ns.push(started.elapsed().as_nanos());
            };
            let mut measure_shared = || {
                let started = Instant::now();
                for _ in 0..CLONES_PER_SAMPLE {
                    black_box(Arc::clone(&shared_path));
                }
                shared_samples_ns.push(started.elapsed().as_nanos());
            };
            if sample_index % 2 == 0 {
                measure_legacy();
                measure_shared();
            } else {
                measure_shared();
                measure_legacy();
            }
        }

        let legacy_p50_ns = nearest_rank_percentile(&legacy_samples_ns, 50);
        let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
        let shared_p50_ns = nearest_rank_percentile(&shared_samples_ns, 50);
        let shared_p95_ns = nearest_rank_percentile(&shared_samples_ns, 95);
        let legacy_ns = join_samples(&legacy_samples_ns);
        let shared_ns = join_samples(&shared_samples_ns);
        let legacy_deep_path_copies = CLONES_PER_SAMPLE * SAMPLE_PAIRS;
        let legacy_copied_path_bytes = path_bytes * legacy_deep_path_copies;

        println!(
            "PREFERENCE_PATH_CACHE_CLONE_BENCH_V1 case={case} path_bytes={path_bytes} \
             clones_per_sample={CLONES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
             legacy_deep_path_copies={legacy_deep_path_copies} shared_deep_path_copies=0 \
             legacy_copied_path_bytes={legacy_copied_path_bytes} shared_copied_path_bytes=0 \
             legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             shared_p50_ns={shared_p50_ns} shared_p95_ns={shared_p95_ns} \
             target_p95_ratio_pct={target_p95_ratio_pct} legacy_ns={legacy_ns} shared_ns={shared_ns}"
        );
        assert!(
            shared_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(target_p95_ratio_pct),
            "{case} shared P95 {shared_p95_ns}ns must be at most {target_p95_ratio_pct}% of legacy P95 {legacy_p95_ns}ns"
        );
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        assert!(!samples.is_empty());
        assert!((1..=100).contains(&percentile));
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let index = (ordered.len() * percentile).div_ceil(100) - 1;
        ordered[index]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
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
