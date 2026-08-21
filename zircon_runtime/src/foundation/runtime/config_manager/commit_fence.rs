use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, TryLockError, Weak};

static PATH_COMMIT_GATES: OnceLock<Mutex<HashMap<PathBuf, Weak<Mutex<PathCommitEpoch>>>>> =
    OnceLock::new();

#[derive(Debug, Default)]
struct PathCommitEpoch {
    current: u64,
}

pub(in crate::foundation::runtime) struct ConfigCommitFence {
    path: PathBuf,
    epoch: u64,
    gate: Arc<Mutex<PathCommitEpoch>>,
    cancelled: AtomicBool,
    commit_active: AtomicBool,
}

impl ConfigCommitFence {
    pub(super) fn register(path: &Path) -> io::Result<Arc<Self>> {
        let path = absolute_path(path);
        let gates = PATH_COMMIT_GATES.get_or_init(|| Mutex::new(HashMap::new()));
        let mut gates = lock(gates);
        let gate = gates.get(&path).and_then(Weak::upgrade).unwrap_or_else(|| {
            let gate = Arc::new(Mutex::new(PathCommitEpoch::default()));
            gates.insert(path.clone(), Arc::downgrade(&gate));
            gate
        });
        let mut state = match gate.try_lock() {
            Ok(state) => state,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    format!(
                        "a config filesystem commit for {} is still in progress",
                        path.display()
                    ),
                ));
            }
        };
        state.current = state.current.wrapping_add(1);
        let epoch = state.current;
        drop(state);
        drop(gates);

        Ok(Arc::new(Self {
            path,
            epoch,
            gate,
            cancelled: AtomicBool::new(false),
            commit_active: AtomicBool::new(false),
        }))
    }

    pub(in crate::foundation::runtime) fn commit<T>(
        &self,
        commit: impl FnOnce() -> io::Result<T>,
    ) -> io::Result<T> {
        let state = lock(&self.gate);
        self.commit_active.store(true, Ordering::Release);
        let active = CommitActiveGuard(&self.commit_active);
        if self.cancelled.load(Ordering::Acquire) || state.current != self.epoch {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                format!(
                    "config commit for {} was cancelled or superseded",
                    self.path.display()
                ),
            ));
        }
        let result = commit();
        drop(active);
        result
    }

    pub(super) fn cancel(&self) -> bool {
        self.cancelled.store(true, Ordering::Release);
        self.commit_active.load(Ordering::Acquire)
    }
}

impl fmt::Debug for ConfigCommitFence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigCommitFence")
            .field("path", &self.path)
            .field("epoch", &self.epoch)
            .field("cancelled", &self.cancelled.load(Ordering::Acquire))
            .field("commit_active", &self.commit_active.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

impl Drop for ConfigCommitFence {
    fn drop(&mut self) {
        if Arc::strong_count(&self.gate) != 1 {
            return;
        }
        let Some(gates) = PATH_COMMIT_GATES.get() else {
            return;
        };
        let mut gates = lock(gates);
        if Arc::strong_count(&self.gate) != 1 {
            return;
        }
        let owns_entry = gates
            .get(&self.path)
            .is_some_and(|registered| registered.as_ptr() == Arc::as_ptr(&self.gate));
        if owns_entry {
            gates.remove(&self.path);
        }
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|directory| directory.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalize_platform_path(normalized)
}

#[cfg(windows)]
fn normalize_platform_path(path: PathBuf) -> PathBuf {
    PathBuf::from(path.to_string_lossy().to_lowercase())
}

#[cfg(not(windows))]
fn normalize_platform_path(path: PathBuf) -> PathBuf {
    path
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct CommitActiveGuard<'a>(&'a AtomicBool);

impl Drop for CommitActiveGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    use super::*;

    const BENCH_PATH_COUNT: usize = 65_536;
    const BENCH_SAMPLE_PAIRS: usize = 21;
    static NEXT_TEST_PATH: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn path_commit_gate_registry_reclaims_only_after_the_last_fence_drops() {
        let path = unique_path("last-owner-reclaim");
        let normalized = absolute_path(&path);
        let first = ConfigCommitFence::register(&path).unwrap();
        let second = ConfigCommitFence::register(&path).unwrap();
        assert!(registry_contains(&normalized));

        drop(first);
        assert!(
            registry_contains(&normalized),
            "a live fence must keep the shared path gate registered"
        );

        drop(second);
        assert!(
            !registry_contains(&normalized),
            "the last fence must reclaim its dead path key"
        );
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn path_commit_gate_registry_reclaim_release_benchmark() {
        let paths = (0..BENCH_PATH_COUNT)
            .map(|index| PathBuf::from(format!("runtime55/config-{index:08}.json")))
            .collect::<Vec<_>>();
        let retained_path_bytes = paths
            .iter()
            .map(|path| path.as_os_str().len())
            .sum::<usize>();
        let mut legacy_ns = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
        let mut legacy_final_entries = 0;
        let mut optimized_final_entries = 0;
        let mut legacy_peak_entries = 0;
        let mut optimized_peak_entries = 0;

        for sample_index in 0..BENCH_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                let sample = measure_legacy(&paths);
                legacy_ns.push(sample.elapsed_ns);
                legacy_final_entries = sample.final_entries;
                legacy_peak_entries = sample.peak_entries;

                let sample = measure_reclaimed(&paths);
                optimized_ns.push(sample.elapsed_ns);
                optimized_final_entries = sample.final_entries;
                optimized_peak_entries = sample.peak_entries;
            } else {
                let sample = measure_reclaimed(&paths);
                optimized_ns.push(sample.elapsed_ns);
                optimized_final_entries = sample.final_entries;
                optimized_peak_entries = sample.peak_entries;

                let sample = measure_legacy(&paths);
                legacy_ns.push(sample.elapsed_ns);
                legacy_final_entries = sample.final_entries;
                legacy_peak_entries = sample.peak_entries;
            }
        }

        assert_eq!(legacy_final_entries, BENCH_PATH_COUNT);
        assert_eq!(legacy_peak_entries, BENCH_PATH_COUNT);
        assert_eq!(optimized_final_entries, 0);
        assert_eq!(optimized_peak_entries, 1);
        let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
        let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
        let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
        let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
        println!(
            "FOUNDATION_PATH_GATE_RECLAIM_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank timing_gate=diagnostic_only path_count={} legacy_final_entries={} optimized_final_entries={} legacy_peak_entries={} optimized_peak_entries={} legacy_retained_path_bytes={} optimized_retained_path_bytes=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
            BENCH_SAMPLE_PAIRS,
            BENCH_PATH_COUNT,
            legacy_final_entries,
            optimized_final_entries,
            legacy_peak_entries,
            optimized_peak_entries,
            retained_path_bytes,
            legacy_p50_ns,
            legacy_p95_ns,
            optimized_p50_ns,
            optimized_p95_ns,
            join_samples(&legacy_ns),
            join_samples(&optimized_ns),
        );
    }

    #[derive(Clone, Copy)]
    struct RegistrySample {
        elapsed_ns: u128,
        final_entries: usize,
        peak_entries: usize,
    }

    fn measure_legacy(paths: &[PathBuf]) -> RegistrySample {
        let started = Instant::now();
        let mut gates = HashMap::<PathBuf, Weak<Mutex<PathCommitEpoch>>>::new();
        let mut peak_entries = 0;
        for path in paths {
            gates.insert(path.clone(), Weak::new());
            peak_entries = peak_entries.max(gates.len());
        }
        black_box(&gates);
        RegistrySample {
            elapsed_ns: started.elapsed().as_nanos(),
            final_entries: gates.len(),
            peak_entries,
        }
    }

    fn measure_reclaimed(paths: &[PathBuf]) -> RegistrySample {
        let started = Instant::now();
        let mut gates = HashMap::<PathBuf, Weak<Mutex<PathCommitEpoch>>>::new();
        let mut peak_entries = 0;
        for path in paths {
            gates.insert(path.clone(), Weak::new());
            peak_entries = peak_entries.max(gates.len());
            gates.remove(path);
        }
        black_box(&gates);
        RegistrySample {
            elapsed_ns: started.elapsed().as_nanos(),
            final_entries: gates.len(),
            peak_entries,
        }
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
        ordered[rank.saturating_sub(1)]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn registry_contains(path: &Path) -> bool {
        PATH_COMMIT_GATES
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(path)
    }

    fn unique_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "zircon-runtime55-{name}-{}-{}",
            std::process::id(),
            NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
