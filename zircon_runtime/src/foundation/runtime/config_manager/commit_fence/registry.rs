use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, TryLockError, Weak};

use super::path_key::absolute_path;

static PATH_COMMIT_GATES: OnceLock<Mutex<HashMap<PathBuf, Weak<Mutex<PathCommitEpoch>>>>> =
    OnceLock::new();

#[derive(Debug, Default)]
pub(super) struct PathCommitEpoch {
    pub(super) current: u64,
}

pub(super) fn register_path_gate(
    path: &Path,
) -> io::Result<(PathBuf, u64, Arc<Mutex<PathCommitEpoch>>)> {
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
    Ok((path, epoch, gate))
}

pub(super) fn reclaim_path_gate(path: &Path, gate: &Arc<Mutex<PathCommitEpoch>>) {
    if Arc::strong_count(gate) != 1 {
        return;
    }
    let Some(gates) = PATH_COMMIT_GATES.get() else {
        return;
    };
    let mut gates = lock(gates);
    if Arc::strong_count(gate) != 1 {
        return;
    }
    let owns_entry = gates
        .get(path)
        .is_some_and(|registered| registered.as_ptr() == Arc::as_ptr(gate));
    if owns_entry {
        gates.remove(path);
    }
}

pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
pub(super) fn contains_path(path: &Path) -> bool {
    lock(PATH_COMMIT_GATES.get_or_init(|| Mutex::new(HashMap::new()))).contains_key(path)
}
