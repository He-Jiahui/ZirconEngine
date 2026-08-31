use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use super::registry::{lock, reclaim_path_gate, register_path_gate, PathCommitEpoch};

pub(in crate::foundation::runtime) struct ConfigCommitFence {
    path: PathBuf,
    epoch: u64,
    gate: Arc<Mutex<PathCommitEpoch>>,
    cancelled: AtomicBool,
    commit_active: AtomicBool,
}

impl ConfigCommitFence {
    pub(in crate::foundation::runtime::config_manager) fn register(
        path: &Path,
    ) -> io::Result<Arc<Self>> {
        let (path, epoch, gate) = register_path_gate(path)?;
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

    pub(in crate::foundation::runtime::config_manager) fn cancel(&self) -> bool {
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
        reclaim_path_gate(&self.path, &self.gate);
    }
}

struct CommitActiveGuard<'a>(&'a AtomicBool);

impl Drop for CommitActiveGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
