use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::time::Duration;

use super::{JobContinuation, JobTerminalObserver};
use crate::core::runtime::tasks::{TaskDescriptor, TaskState};

pub(crate) struct TaskNode {
    descriptor: Option<TaskDescriptor>,
    graph_owner: Option<Weak<()>>,
    inner: Mutex<TaskNodeState>,
    complete: Condvar,
    terminal_observer_panics: AtomicUsize,
    public_handle_count: AtomicUsize,
}

pub(in crate::core::runtime::tasks) struct TaskNodeState {
    pub(super) lifecycle: TaskState,
    pub(super) is_cancelled: bool,
    pub(in crate::core::runtime::tasks) cancellation_requested: bool,
    pub(in crate::core::runtime::tasks) cancellation_acknowledged: bool,
    pub(super) dependency_continuations_published: bool,
    pub(super) terminal_observer_delivery_active: bool,
    pub(super) panic_message: Option<Arc<str>>,
    pub(super) remaining_dependencies: usize,
    pub(super) dependents: Vec<JobContinuation>,
    pub(super) terminal_observers: Vec<JobTerminalObserver>,
}

impl TaskNode {
    pub(super) fn new(
        descriptor: Option<TaskDescriptor>,
        remaining_dependencies: usize,
        graph_owner: Option<Weak<()>>,
    ) -> Self {
        Self {
            descriptor,
            graph_owner,
            inner: Mutex::new(TaskNodeState {
                lifecycle: TaskState::Pending,
                is_cancelled: false,
                cancellation_requested: false,
                cancellation_acknowledged: false,
                dependency_continuations_published: false,
                terminal_observer_delivery_active: false,
                panic_message: None,
                remaining_dependencies,
                dependents: Vec::new(),
                terminal_observers: Vec::new(),
            }),
            complete: Condvar::new(),
            terminal_observer_panics: AtomicUsize::new(0),
            public_handle_count: AtomicUsize::new(0),
        }
    }

    pub(in crate::core::runtime::tasks) fn lock_inner(&self) -> MutexGuard<'_, TaskNodeState> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    pub(super) fn poison_inner_for_test(&self) -> ! {
        let _guard = self.inner.lock().unwrap();
        panic!("poison canonical task node state")
    }

    pub(super) fn wait_inner<'a>(
        &self,
        inner: MutexGuard<'a, TaskNodeState>,
    ) -> MutexGuard<'a, TaskNodeState> {
        self.complete
            .wait(inner)
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn wait_inner_timeout<'a>(
        &self,
        inner: MutexGuard<'a, TaskNodeState>,
        timeout: Duration,
    ) -> MutexGuard<'a, TaskNodeState> {
        self.complete
            .wait_timeout(inner, timeout)
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .0
    }

    pub(super) fn notify_terminal(&self) {
        self.complete.notify_all();
    }

    pub(super) fn release_dependency_continuations(&self) -> Option<Vec<JobTerminalObserver>> {
        let mut inner = self.lock_inner();
        inner.dependency_continuations_published = true;
        Self::take_terminal_observer_batch(&mut inner)
    }

    pub(super) fn finish_terminal_observer_batch(&self) -> Option<Vec<JobTerminalObserver>> {
        let mut inner = self.lock_inner();
        inner.terminal_observer_delivery_active = false;
        Self::take_terminal_observer_batch(&mut inner)
    }

    // delivery_active 将 observer 通知批次串行化；当前批次发布完成前不会重入下一批 late observer。
    pub(super) fn take_terminal_observer_batch(
        inner: &mut TaskNodeState,
    ) -> Option<Vec<JobTerminalObserver>> {
        if inner.terminal_observer_delivery_active || inner.terminal_observers.is_empty() {
            None
        } else {
            inner.terminal_observer_delivery_active = true;
            Some(std::mem::take(&mut inner.terminal_observers))
        }
    }

    pub(super) fn run_terminal_observer(&self, observer: JobTerminalObserver) {
        if catch_unwind(AssertUnwindSafe(observer)).is_err() {
            self.terminal_observer_panics
                .fetch_add(1, Ordering::Release);
        }
    }

    pub(super) fn terminal_observer_panic_count(&self) -> usize {
        self.terminal_observer_panics.load(Ordering::Acquire)
    }

    pub(in crate::core::runtime::tasks) fn descriptor(&self) -> &TaskDescriptor {
        self.descriptor
            .as_ref()
            .expect("descriptor-led task nodes must retain their admission descriptor")
    }

    pub(in crate::core::runtime::tasks) fn belongs_to_graph(&self, graph_owner: &Arc<()>) -> bool {
        self.graph_owner
            .as_ref()
            .and_then(|owner| owner.upgrade())
            .is_some_and(|owner| Arc::ptr_eq(&owner, graph_owner))
    }

    pub(in crate::core::runtime::tasks) fn request_cancellation(&self) {
        self.lock_inner().cancellation_requested = true;
    }

    pub(in crate::core::runtime::tasks) fn retain_public_handle(&self) {
        let prior = self.public_handle_count.fetch_add(1, Ordering::Relaxed);
        debug_assert!(prior < usize::MAX, "task handle count overflow");
    }

    pub(in crate::core::runtime::tasks) fn release_public_handle(&self) -> bool {
        let prior = self.public_handle_count.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(prior > 0, "task handle count underflow");
        prior == 1
    }

    #[cfg(test)]
    pub(in crate::core::runtime::tasks) fn identity(node: &Arc<Self>) -> usize {
        Arc::as_ptr(node) as usize
    }

    #[cfg(test)]
    pub(in crate::core::runtime::tasks) fn graph_owner_weak_count(&self) -> usize {
        self.graph_owner
            .as_ref()
            .map_or(0, std::sync::Weak::weak_count)
    }
}
