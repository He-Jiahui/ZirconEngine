use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Clone)]
pub(super) struct OwnedWorkerThreads {
    expected_worker_count: Arc<AtomicUsize>,
    shared: Arc<OwnedWorkerThreadsShared>,
}

struct OwnedWorkerThreadsShared {
    state: Mutex<OwnedWorkerThreadsState>,
    changed: Condvar,
}

struct OwnedWorkerThreadsState {
    handles: Vec<JoinHandle<()>>,
    exited_worker_count: usize,
    joined_worker_count: usize,
}

struct WorkerExitReceipt {
    shared: Arc<OwnedWorkerThreadsShared>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct OwnedWorkerThreadsCensus {
    pub expected_worker_count: usize,
    pub exited_worker_count: usize,
    pub joined_worker_count: usize,
}

impl OwnedWorkerThreadsCensus {
    pub const fn all_joined(self) -> bool {
        self.joined_worker_count == self.expected_worker_count
    }
}

impl OwnedWorkerThreads {
    pub(super) fn new(expected_worker_count: usize) -> Self {
        Self {
            expected_worker_count: Arc::new(AtomicUsize::new(expected_worker_count)),
            shared: Arc::new(OwnedWorkerThreadsShared {
                state: Mutex::new(OwnedWorkerThreadsState {
                    handles: Vec::with_capacity(expected_worker_count),
                    exited_worker_count: 0,
                    joined_worker_count: 0,
                }),
                changed: Condvar::new(),
            }),
        }
    }

    pub(super) fn set_expected_worker_count(&self, expected_worker_count: usize) {
        self.expected_worker_count
            .store(expected_worker_count, Ordering::Release);
    }

    pub(super) fn spawn(
        &self,
        name: Option<String>,
        stack_size: Option<usize>,
        worker: impl FnOnce() + Send + 'static,
    ) -> io::Result<()> {
        let mut thread_builder = thread::Builder::new();
        if let Some(name) = name {
            thread_builder = thread_builder.name(name);
        }
        if let Some(stack_size) = stack_size {
            thread_builder = thread_builder.stack_size(stack_size);
        }

        let exit_receipt = Arc::clone(&self.shared);
        let handle = thread_builder.spawn(move || {
            let _exit_receipt = WorkerExitReceipt {
                shared: exit_receipt,
            };
            worker();
        })?;
        self.shared.lock_state().handles.push(handle);
        Ok(())
    }

    pub(super) fn wait_and_join(&self, timeout: Duration) -> OwnedWorkerThreadsCensus {
        let expected_worker_count = self.expected_worker_count();
        let mut state = self.shared.lock_state();
        if state.exited_worker_count < expected_worker_count {
            let (state_after_wait, _) = self
                .shared
                .changed
                .wait_timeout_while(state, timeout, |state| {
                    state.exited_worker_count < expected_worker_count
                })
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = state_after_wait;
        }
        if state.exited_worker_count < expected_worker_count {
            return self.census_from(&state);
        }

        let handles = std::mem::take(&mut state.handles);
        drop(state);
        let mut joined_worker_count = 0;
        for handle in handles {
            let _ = handle.join();
            joined_worker_count += 1;
        }

        let mut state = self.shared.lock_state();
        state.joined_worker_count = state
            .joined_worker_count
            .saturating_add(joined_worker_count);
        self.census_from(&state)
    }

    pub(super) fn join_spawned_workers(&self) -> OwnedWorkerThreadsCensus {
        let spawned_worker_count = self.shared.lock_state().handles.len();
        let mut state = self.shared.lock_state();
        while state.exited_worker_count < spawned_worker_count {
            state = self
                .shared
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        let handles = std::mem::take(&mut state.handles);
        drop(state);

        let joined_worker_count = handles
            .into_iter()
            .map(|handle| {
                let _ = handle.join();
                1_usize
            })
            .sum::<usize>();
        let mut state = self.shared.lock_state();
        state.joined_worker_count = state
            .joined_worker_count
            .saturating_add(joined_worker_count);
        self.census_from(&state)
    }

    pub(super) fn census(&self) -> OwnedWorkerThreadsCensus {
        self.census_from(&self.shared.lock_state())
    }

    fn census_from(&self, state: &OwnedWorkerThreadsState) -> OwnedWorkerThreadsCensus {
        OwnedWorkerThreadsCensus {
            expected_worker_count: self.expected_worker_count(),
            exited_worker_count: state.exited_worker_count,
            joined_worker_count: state.joined_worker_count,
        }
    }

    fn expected_worker_count(&self) -> usize {
        self.expected_worker_count.load(Ordering::Acquire)
    }
}

impl Drop for WorkerExitReceipt {
    fn drop(&mut self) {
        self.shared.record_exit();
    }
}

impl OwnedWorkerThreadsShared {
    fn record_exit(&self) {
        let mut state = self.lock_state();
        state.exited_worker_count = state.exited_worker_count.saturating_add(1);
        self.changed.notify_all();
    }

    fn lock_state(&self) -> MutexGuard<'_, OwnedWorkerThreadsState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
