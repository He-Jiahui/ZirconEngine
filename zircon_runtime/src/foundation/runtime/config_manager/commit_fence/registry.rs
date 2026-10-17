use std::collections::BTreeMap;
use std::io;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
#[cfg(test)]
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, TryLockError, Weak};

use crate::asset::project::ResolvedProjectPathIdentity;
use crate::core::{CoreError, CoreWeak};

#[cfg(test)]
use super::path_key::physical_path;

static PATH_COMMIT_GATES: OnceLock<Mutex<BTreeMap<ResolvedProjectPathIdentity, PathEntry>>> =
    OnceLock::new();

#[derive(Debug, Default)]
pub(super) struct PathCommitEpoch {
    pub(super) current: u64,
}

#[derive(Default)]
struct PathEntry {
    gate: Weak<Mutex<PathCommitEpoch>>,
    owner: Weak<ConfigOwnerLease>,
}

/// A live fence owns the lease even after its Core disappears or it is cancelled.
/// The weak Core identifies diagnostics only; it never determines lease expiry.
pub(super) struct ConfigOwnerLease {
    core: CoreWeak,
    path: ResolvedProjectPathIdentity,
    active: AtomicBool,
}

pub(super) struct PendingOwnerLease {
    pub(super) lease: Arc<ConfigOwnerLease>,
    pub(super) prepared: bool,
}

/// Reserve before taking the path gate, so a busy live writer still reports owner conflict.
/// Same-Core independent managers are also conflicts; only cloning the manager shares a writer.
pub(super) fn reserve_path_owner(
    path: &ResolvedProjectPathIdentity,
    core: CoreWeak,
) -> Result<PendingOwnerLease, CoreError> {
    // Own all strong lease references outside the registry-guard scope. The final
    // ConfigOwnerLease Drop may acquire this registry, including on conflict/unwind.
    let candidate = Arc::new(ConfigOwnerLease {
        core,
        path: path.clone(),
        active: AtomicBool::new(true),
    });
    let previous_owner;
    let result = {
        let gates = PATH_COMMIT_GATES.get_or_init(|| Mutex::new(BTreeMap::new()));
        let mut gates = lock(gates);
        let entry = gates.entry(path.clone()).or_default();
        previous_owner = entry.owner.upgrade();
        if let Some(owner) = previous_owner
            .as_ref()
            .filter(|owner| owner.active.load(Ordering::Acquire))
        {
            Err(CoreError::ConfigOwnerConflict {
                path: path.operation_path().display().to_string(),
                same_runtime: Weak::ptr_eq(&owner.core.inner, &candidate.core.inner),
            })
        } else {
            entry.owner = Arc::downgrade(&candidate);
            Ok(PendingOwnerLease {
                lease: Arc::clone(&candidate),
                prepared: false,
            })
        }
    };
    drop(previous_owner);
    drop(candidate);
    result
}

impl Drop for PendingOwnerLease {
    fn drop(&mut self) {
        if !self.prepared {
            // This outer guard runs only after the path guard has been released.
            self.lease.active.store(false, Ordering::Release);
            reclaim_owner_lease(&self.lease);
        }
    }
}

impl Drop for ConfigOwnerLease {
    fn drop(&mut self) {
        reclaim_owner_lease(self);
    }
}

fn reclaim_owner_lease(owner: &ConfigOwnerLease) {
    let Some(gates) = PATH_COMMIT_GATES.get() else {
        return;
    };
    let mut gates = lock(gates);
    let owns_empty_entry = gates.get(&owner.path).is_some_and(|entry| {
        entry.owner.as_ptr() == owner as *const ConfigOwnerLease && entry.gate.strong_count() == 0
    });
    if owns_empty_entry {
        gates.remove(&owner.path);
    }
}

/// Prepare under the path gate and publish the epoch only after preparation succeeds.
/// The physical identity is already captured; registry lookup never changes the I/O address.
/// The global registry lock is released before the path gate, recovery, or worker startup.
/// Preparation must not recursively commit, or drop/join a worker that needs this gate.
pub(super) fn initialize_path_gate<T, E>(
    path: ResolvedProjectPathIdentity,
    initialize: impl FnOnce(
        ResolvedProjectPathIdentity,
        u64,
        Arc<Mutex<PathCommitEpoch>>,
    ) -> Result<T, E>,
) -> io::Result<Result<T, E>> {
    let gate = {
        let gates = PATH_COMMIT_GATES.get_or_init(|| Mutex::new(BTreeMap::new()));
        let mut gates = lock(gates);
        let entry = gates.entry(path.clone()).or_default();
        entry.gate.upgrade().unwrap_or_else(|| {
            let gate = Arc::new(Mutex::new(PathCommitEpoch::default()));
            entry.gate = Arc::downgrade(&gate);
            gate
        })
    };
    let lease = PathGateLease { path, gate };
    let mut state = match lease.gate.try_lock() {
        Ok(state) => state,
        Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(TryLockError::WouldBlock) => {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                format!(
                    "a config filesystem commit for {} is still in progress",
                    lease.path.operation_path().display()
                ),
            ));
        }
    };
    let epoch = state.current.checked_add(1).ok_or_else(|| {
        io::Error::other(format!(
            "config filesystem commit epoch exhausted for {}",
            lease.path.operation_path().display()
        ))
    })?;
    // Catch while the guard is still owned, then unlock outside unwinding. Existing owners
    // keep an unpoisoned gate and their unchanged epoch when preparation panics.
    let initialized = catch_unwind(AssertUnwindSafe(|| {
        initialize(lease.path.clone(), epoch, Arc::clone(&lease.gate))
    }));
    if matches!(&initialized, Ok(Ok(_))) {
        state.current = epoch;
    }
    drop(state);
    drop(lease);
    match initialized {
        Ok(result) => Ok(result),
        Err(payload) => resume_unwind(payload),
    }
}

#[cfg(test)]
pub(super) fn register_path_gate(
    path: &Path,
) -> io::Result<(
    ResolvedProjectPathIdentity,
    u64,
    Arc<Mutex<PathCommitEpoch>>,
)> {
    initialize_path_gate(physical_path(path)?, |path, epoch, gate| {
        Ok((path, epoch, gate))
    })?
}

struct PathGateLease {
    path: ResolvedProjectPathIdentity,
    gate: Arc<Mutex<PathCommitEpoch>>,
}

impl Drop for PathGateLease {
    fn drop(&mut self) {
        reclaim_path_gate(&self.path, &self.gate);
    }
}

pub(super) fn reclaim_path_gate(
    path: &ResolvedProjectPathIdentity,
    gate: &Arc<Mutex<PathCommitEpoch>>,
) {
    if Arc::strong_count(gate) != 1 {
        return;
    }
    let Some(gates) = PATH_COMMIT_GATES.get() else {
        return;
    };
    let observed_owner;
    {
        let mut gates = lock(gates);
        if Arc::strong_count(gate) != 1 {
            return;
        }
        let entry = gates.get(path);
        observed_owner = entry.and_then(|entry| entry.owner.upgrade());
        let live_owner = observed_owner
            .as_ref()
            .is_some_and(|owner| owner.active.load(Ordering::Acquire));
        let owns_entry = entry.is_some_and(|entry| entry.gate.as_ptr() == Arc::as_ptr(gate));
        // A newly reserved owner may not have attached its gate yet. An older failed
        // candidate must not remove that reservation when its last gate drops.
        if owns_entry && !live_owner {
            gates.remove(path);
        }
    }
    // Never drop an upgraded strong lease reference while holding the registry mutex.
    drop(observed_owner);
}

pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
pub(super) fn contains_path(path: &ResolvedProjectPathIdentity) -> bool {
    lock(PATH_COMMIT_GATES.get_or_init(|| Mutex::new(BTreeMap::new()))).contains_key(path)
}
