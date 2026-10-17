use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::error::HubError;
use crate::process::editor_child_receipt::EditorChildTerminalReceipt;
use crate::process::SupervisedChild;

const EDITOR_CHILD_REAP_INTERVAL: Duration = Duration::from_millis(250);
const EDITOR_CHILD_CLEANUP_WAIT: Duration = Duration::from_secs(2);

pub(crate) struct EditorChildReaper {
    inner: Arc<EditorChildReaperInner>,
    worker: Arc<Mutex<Option<JoinHandle<()>>>>,
}

struct EditorChildReaperInner {
    children: Mutex<Vec<TrackedChild>>,
    shutdown_deadline: Mutex<Option<Instant>>,
    wake: Condvar,
    handle_count: AtomicUsize,
    shutdown_requested: AtomicBool,
    on_terminal: Box<dyn Fn(EditorChildTerminalReceipt) + Send + Sync>,
}

struct TrackedChild {
    attempt_id: u64,
    child: SupervisedChild,
}

impl EditorChildReaper {
    pub(crate) fn start() -> Result<Self, HubError> {
        Self::start_with_observer(|_| {})
    }

    pub(crate) fn start_with_observer(
        on_terminal: impl Fn(EditorChildTerminalReceipt) + Send + Sync + 'static,
    ) -> Result<Self, HubError> {
        let inner = Arc::new(EditorChildReaperInner {
            children: Mutex::new(Vec::new()),
            shutdown_deadline: Mutex::new(None),
            wake: Condvar::new(),
            handle_count: AtomicUsize::new(1),
            shutdown_requested: AtomicBool::new(false),
            on_terminal: Box::new(on_terminal),
        });
        let weak_inner = Arc::downgrade(&inner);
        let worker = thread::Builder::new()
            .name("zircon-hub-editor-child-reaper".to_string())
            .spawn(move || run_reaper(weak_inner))?;
        Ok(Self {
            inner,
            worker: Arc::new(Mutex::new(Some(worker))),
        })
    }

    pub(crate) fn register(
        &self,
        attempt_id: u64,
        mut child: SupervisedChild,
    ) -> Result<(), HubError> {
        let mut children = lock_children(&self.inner);
        if self.inner.shutdown_requested.load(Ordering::Acquire) {
            drop(children);
            let process_id = child.id();
            let cleanup_error = child
                .terminate_tree_and_reap_until(self.inner.cleanup_deadline())
                .err()
                .map(|error| error.to_string());
            (self.inner.on_terminal)(EditorChildTerminalReceipt::stopped_by_hub(
                attempt_id,
                process_id,
                cleanup_error,
            ));
            return Ok(());
        }
        children.push(TrackedChild { attempt_id, child });
        drop(children);
        self.inner.wake.notify_one();
        Ok(())
    }

    pub(crate) fn shutdown_and_join_until(&self, deadline: Instant) -> Result<(), HubError> {
        let deadline = self.request_shutdown_until(deadline);
        loop {
            let finished = self
                .worker
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_ref()
                .is_none_or(JoinHandle::is_finished);
            if finished {
                let worker = self
                    .worker
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .take();
                if let Some(worker) = worker {
                    worker.join().map_err(|_| {
                        HubError::message("Hub Editor child reaper panicked during shutdown")
                    })?;
                }
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(HubError::message(
                    "Hub Editor child reaper did not finish before its shutdown deadline",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub(crate) fn request_shutdown_until(&self, deadline: Instant) -> Instant {
        self.inner.request_shutdown_until(deadline)
    }

    pub(crate) fn cancel_before_ready(
        &self,
        attempt_id: u64,
        child: SupervisedChild,
    ) -> Result<(), HubError> {
        self.finish_before_ready(attempt_id, child, false)
    }

    pub(crate) fn fail_before_ready(
        &self,
        attempt_id: u64,
        child: SupervisedChild,
    ) -> Result<(), HubError> {
        self.finish_before_ready(attempt_id, child, true)
    }

    fn finish_before_ready(
        &self,
        attempt_id: u64,
        mut child: SupervisedChild,
        failed: bool,
    ) -> Result<(), HubError> {
        let process_id = child.id();
        let cleanup = child.terminate_tree_and_reap_until(self.inner.cleanup_deadline());
        let cleanup_error = cleanup.as_ref().err().map(ToString::to_string);
        let receipt = if failed {
            EditorChildTerminalReceipt::failed_before_ready(attempt_id, process_id, cleanup_error)
        } else {
            EditorChildTerminalReceipt::stopped_before_ready(attempt_id, process_id, cleanup_error)
        };
        (self.inner.on_terminal)(receipt);
        cleanup
    }
}

impl EditorChildReaperInner {
    fn request_shutdown_until(&self, deadline: Instant) -> Instant {
        let mut stored = self
            .shutdown_deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let deadline = stored.map_or(deadline, |previous| previous.min(deadline));
        *stored = Some(deadline);
        drop(stored);
        self.shutdown_requested.store(true, Ordering::Release);
        self.wake.notify_all();
        deadline
    }

    fn cleanup_deadline(&self) -> Instant {
        self.shutdown_deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .unwrap_or_else(|| Instant::now() + EDITOR_CHILD_CLEANUP_WAIT)
    }
}

impl Clone for EditorChildReaper {
    fn clone(&self) -> Self {
        self.inner.handle_count.fetch_add(1, Ordering::Relaxed);
        Self {
            inner: Arc::clone(&self.inner),
            worker: Arc::clone(&self.worker),
        }
    }
}

impl Drop for EditorChildReaper {
    fn drop(&mut self) {
        if self.inner.handle_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.inner
                .request_shutdown_until(Instant::now() + EDITOR_CHILD_CLEANUP_WAIT);
        }
    }
}

fn run_reaper(inner: Weak<EditorChildReaperInner>) {
    loop {
        let Some(inner) = inner.upgrade() else {
            return;
        };
        let mut children = lock_children(&inner);
        while children.is_empty()
            && inner.handle_count.load(Ordering::Acquire) != 0
            && !inner.shutdown_requested.load(Ordering::Acquire)
        {
            children = inner
                .wake
                .wait(children)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if inner.handle_count.load(Ordering::Acquire) == 0
            || inner.shutdown_requested.load(Ordering::Acquire)
        {
            let remaining = std::mem::take(&mut *children);
            drop(children);
            let deadline = inner.cleanup_deadline();
            for tracked in remaining {
                let receipt = stop_tracked_child(tracked, deadline);
                (inner.on_terminal)(receipt);
            }
            return;
        }
        let finished = reap_finished_children(&mut children);
        if !children.is_empty() {
            let (guard, _) = inner
                .wake
                .wait_timeout(children, EDITOR_CHILD_REAP_INTERVAL)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            drop(guard);
        } else {
            drop(children);
        }
        for (tracked, status) in finished {
            let receipt = reap_tracked_child(tracked, status, inner.cleanup_deadline());
            (inner.on_terminal)(receipt);
        }
    }
}

fn reap_finished_children(children: &mut Vec<TrackedChild>) -> Vec<(TrackedChild, ExitStatus)> {
    let mut finished = Vec::new();
    let mut index = 0;
    while index < children.len() {
        match children[index].child.try_wait() {
            Ok(Some(status)) => finished.push((children.swap_remove(index), status)),
            Ok(None) => index += 1,
            Err(error) => {
                eprintln!(
                    "zircon_hub: failed to query Editor child {} for reaping: {error}",
                    children[index].child.id()
                );
                index += 1;
            }
        }
    }
    finished
}

fn reap_tracked_child(
    mut tracked: TrackedChild,
    status: ExitStatus,
    deadline: Instant,
) -> EditorChildTerminalReceipt {
    let cleanup_error = tracked
        .child
        .terminate_tree_and_reap_until(deadline)
        .err()
        .map(|error| error.to_string());
    EditorChildTerminalReceipt::exited(
        tracked.attempt_id,
        tracked.child.id(),
        status,
        cleanup_error,
    )
}

fn stop_tracked_child(mut tracked: TrackedChild, deadline: Instant) -> EditorChildTerminalReceipt {
    let cleanup_error = tracked
        .child
        .terminate_tree_and_reap_until(deadline)
        .err()
        .map(|error| error.to_string());
    EditorChildTerminalReceipt::stopped_by_hub(
        tracked.attempt_id,
        tracked.child.id(),
        cleanup_error,
    )
}

fn lock_children(inner: &EditorChildReaperInner) -> MutexGuard<'_, Vec<TrackedChild>> {
    inner.children.lock().unwrap_or_else(|poisoned| {
        eprintln!("zircon_hub: Editor child reaper lock was poisoned; recovering tracked children");
        poisoned.into_inner()
    })
}

#[cfg(test)]
#[path = "tests/editor_child_reaper.rs"]
mod tests;

#[cfg(test)]
#[path = "editor_child_reaper/tests/cases.rs"]
mod terminal_tests;
