use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub(super) struct UiAssetWatchPendingPath {
    pub(super) path: PathBuf,
    first_seen_at: Instant,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct UiAssetWatchIngressSnapshot {
    pub(super) pending_path_count: usize,
    pub(super) received_path_count: u64,
    pub(super) coalesced_path_count: u64,
    pub(super) overflow_count: u64,
    pub(super) overflow_pending: bool,
    pub(super) oldest_pending_age: Duration,
}

#[derive(Clone)]
pub(super) struct UiAssetWatchIngressHandle {
    state: Arc<Mutex<UiAssetWatchIngress>>,
}

struct UiAssetWatchIngress {
    max_pending_paths: usize,
    pending_paths: VecDeque<UiAssetWatchPendingPath>,
    pending_path_set: HashSet<PathBuf>,
    received_path_count: u64,
    coalesced_path_count: u64,
    overflow_count: u64,
    overflow_pending: bool,
}

impl UiAssetWatchIngressHandle {
    pub(super) fn new(max_pending_paths: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(UiAssetWatchIngress {
                max_pending_paths,
                pending_paths: VecDeque::with_capacity(max_pending_paths),
                pending_path_set: HashSet::with_capacity(max_pending_paths),
                received_path_count: 0,
                coalesced_path_count: 0,
                overflow_count: 0,
                overflow_pending: false,
            })),
        }
    }

    pub(super) fn record_paths(&self, paths: impl IntoIterator<Item = PathBuf>) {
        self.record_paths_at(paths, Instant::now());
    }

    pub(super) fn take_overflow(&self) -> bool {
        let mut state = self.lock_state();
        if !state.overflow_pending {
            return false;
        }
        state.overflow_pending = false;
        state.pending_paths.clear();
        state.pending_path_set.clear();
        true
    }

    pub(super) fn drain_paths(&self, limit: usize) -> Vec<UiAssetWatchPendingPath> {
        let mut state = self.lock_state();
        let count = limit.min(state.pending_paths.len());
        let mut drained = Vec::with_capacity(count);
        for _ in 0..count {
            let Some(pending) = state.pending_paths.pop_front() else {
                break;
            };
            let _ = state.pending_path_set.remove(&pending.path);
            drained.push(pending);
        }
        drained
    }

    pub(super) fn restore_paths_front(&self, paths: Vec<UiAssetWatchPendingPath>) {
        if paths.is_empty() {
            return;
        }
        let mut state = self.lock_state();
        let mut restored = Vec::new();
        for pending in paths {
            if state.pending_path_set.contains(&pending.path) {
                continue;
            }
            if state.pending_paths.len() + restored.len() >= state.max_pending_paths {
                state.mark_overflow();
                continue;
            }
            let _ = state.pending_path_set.insert(pending.path.clone());
            restored.push(pending);
        }
        for pending in restored.into_iter().rev() {
            state.pending_paths.push_front(pending);
        }
    }

    pub(super) fn snapshot(&self, now: Instant) -> UiAssetWatchIngressSnapshot {
        let state = self.lock_state();
        UiAssetWatchIngressSnapshot {
            pending_path_count: state.pending_paths.len(),
            received_path_count: state.received_path_count,
            coalesced_path_count: state.coalesced_path_count,
            overflow_count: state.overflow_count,
            overflow_pending: state.overflow_pending,
            oldest_pending_age: state
                .pending_paths
                .front()
                .map(|pending| now.saturating_duration_since(pending.first_seen_at))
                .unwrap_or_default(),
        }
    }

    pub(super) fn record_paths_at(
        &self,
        paths: impl IntoIterator<Item = PathBuf>,
        observed_at: Instant,
    ) {
        let mut state = self.lock_state();
        for path in paths {
            state.received_path_count = state.received_path_count.saturating_add(1);
            if state.pending_path_set.contains(&path) {
                state.coalesced_path_count = state.coalesced_path_count.saturating_add(1);
                continue;
            }
            if state.pending_paths.len() >= state.max_pending_paths {
                state.mark_overflow();
                continue;
            }
            let _ = state.pending_path_set.insert(path.clone());
            state.pending_paths.push_back(UiAssetWatchPendingPath {
                path,
                first_seen_at: observed_at,
            });
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, UiAssetWatchIngress> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl UiAssetWatchIngress {
    fn mark_overflow(&mut self) {
        if self.overflow_pending {
            return;
        }
        self.overflow_pending = true;
        self.overflow_count = self.overflow_count.saturating_add(1);
    }
}
