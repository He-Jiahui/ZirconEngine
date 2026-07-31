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
