mod owned_workers;

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, TryLockError, Weak};
use std::time::{Duration, Instant};

use rayon::ThreadPool;

use self::owned_workers::OwnedWorkerThreads;
use super::{TaskPoolDescriptor, TaskPoolKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TaskPoolYield {
    Executed,
    Idle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskPoolBuildError {
    kind: TaskPoolKind,
    thread_name: String,
    message: String,
}

impl fmt::Display for TaskPoolBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "failed to build {:?} task pool `{}`: {}",
            self.kind, self.thread_name, self.message
        )
    }
}

impl std::error::Error for TaskPoolBuildError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TaskPoolShutdownCensus {
    pub kind: TaskPoolKind,
    pub active_submission_count: usize,
    pub expected_worker_count: usize,
    pub exited_worker_count: usize,
    pub joined_worker_count: usize,
    pub termination_signalled: bool,
}

/// An execution handle for one task-pool domain.
///
/// The instance returned by `try_new` retains the sole strong pool owner.
/// Clones are weak execution handles so schedulers cannot keep workers alive
/// after the owning runtime begins shutdown.
pub struct TaskPool {
    descriptor: Arc<TaskPoolDescriptor>,
    parallelism: usize,
    pool: Weak<OwnedRayonPool>,
    submission_tracker: Arc<TaskPoolSubmissionTracker>,
    termination_signalled: Arc<AtomicBool>,
    worker_threads: OwnedWorkerThreads,
    owner: Option<Arc<TaskPoolOwner>>,
}

pub(in crate::core::runtime) struct TaskPoolSubmission {
    pool: Weak<OwnedRayonPool>,
    tracker: Arc<TaskPoolSubmissionTracker>,
}

struct TaskPoolOwner {
    pool: Mutex<Option<Arc<OwnedRayonPool>>>,
    shutdown_gate: Mutex<()>,
}

struct OwnedRayonPool {
    pool: Option<ThreadPool>,
    submission_tracker: Arc<TaskPoolSubmissionTracker>,
    termination_signalled: Arc<AtomicBool>,
}

struct TaskPoolSubmissionTracker {
    state: AtomicUsize,
    quiescent_wait: Mutex<()>,
    submissions_quiescent: Condvar,
}

const TASK_POOL_SUBMISSIONS_CLOSED: usize = 1 << (usize::BITS - 1);
const TASK_POOL_ACTIVE_SUBMISSION_MASK: usize = TASK_POOL_SUBMISSIONS_CLOSED - 1;

impl TaskPool {
    pub fn new(descriptor: TaskPoolDescriptor) -> Self {
        Self::try_new(descriptor).unwrap_or_else(|error| panic!("{error}"))
    }

    pub fn try_new(descriptor: TaskPoolDescriptor) -> Result<Self, TaskPoolBuildError> {
        let parallelism = descriptor
            .worker_threads
            .unwrap_or_else(default_parallelism)
            .max(1);
        let descriptor = Arc::new(descriptor);
        let thread_name = descriptor.thread_name.clone();
        let worker_threads = OwnedWorkerThreads::new(parallelism);
        let worker_spawn = worker_threads.clone();
        let builder = rayon::ThreadPoolBuilder::new()
            .thread_name(move |index| format!("{thread_name}-{index}"))
            .num_threads(parallelism)
            .spawn_handler(move |worker| {
                let name = worker.name().map(str::to_owned);
                let stack_size = worker.stack_size();
                worker_spawn.spawn(name, stack_size, move || worker.run())
            });
        let pool = match builder.build() {
            Ok(pool) => pool,
            Err(error) => {
                let _ = worker_threads.join_spawned_workers();
                return Err(TaskPoolBuildError {
                    kind: descriptor.kind,
                    thread_name: descriptor.thread_name.clone(),
                    message: error.to_string(),
                });
            }
        };
        let parallelism = pool.current_num_threads();
        worker_threads.set_expected_worker_count(parallelism);

        let termination_signalled = Arc::new(AtomicBool::new(false));
        let submission_tracker = Arc::new(TaskPoolSubmissionTracker {
            state: AtomicUsize::new(0),
            quiescent_wait: Mutex::new(()),
            submissions_quiescent: Condvar::new(),
        });
        let owned_pool = Arc::new(OwnedRayonPool {
            pool: Some(pool),
            submission_tracker: Arc::clone(&submission_tracker),
            termination_signalled: Arc::clone(&termination_signalled),
        });
        let weak_pool = Arc::downgrade(&owned_pool);
        let owner = Arc::new(TaskPoolOwner {
            pool: Mutex::new(Some(owned_pool)),
            shutdown_gate: Mutex::new(()),
        });

        Ok(Self {
            descriptor,
            parallelism,
            pool: weak_pool,
            submission_tracker: Arc::clone(&submission_tracker),
            termination_signalled,
            worker_threads,
            owner: Some(owner),
        })
    }

    pub fn kind(&self) -> TaskPoolKind {
        self.descriptor.kind
    }

    pub fn descriptor(&self) -> &TaskPoolDescriptor {
        &self.descriptor
    }

    pub fn parallelism(&self) -> usize {
        self.parallelism
    }

    pub fn shares_execution_owner_with(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.pool, &other.pool)
    }

    pub(crate) fn is_current_worker(&self) -> bool {
        self.pool
            .upgrade()
            .is_some_and(|pool| pool.pool().current_thread_index().is_some())
    }

    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        self.submission_or_panic().spawn(task);
    }

    pub fn install<R: Send>(&self, task: impl FnOnce() -> R + Send) -> R {
        self.submission_or_panic().install(task)
    }

    pub(crate) fn in_place_scope<'scope, OP, R>(&self, task: OP) -> R
    where
        OP: FnOnce(&rayon::Scope<'scope>) -> R,
    {
        self.submission_or_panic().in_place_scope(task)
    }

    pub fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.submission_or_panic().join(task_a, task_b)
    }

    pub(super) fn close_admission(&self) {
        self.submission_tracker.close_admission();
    }

    pub(super) fn try_acquire_submission(&self) -> Option<TaskPoolSubmission> {
        self.pool
            .upgrade()
            .and_then(OwnedRayonPool::try_acquire_submission)
    }

    pub(super) fn try_acquire_continuation(&self) -> Option<TaskPoolSubmission> {
        self.pool
            .upgrade()
            .and_then(OwnedRayonPool::try_acquire_continuation)
    }

    pub(super) fn close_and_join(&self, timeout: Duration) -> TaskPoolShutdownCensus {
        let called_from_owned_worker = self
            .pool
            .upgrade()
            .is_some_and(|pool| pool.pool().current_thread_index().is_some());
        self.close_admission();
        let started_at = Instant::now();
        if called_from_owned_worker {
            return self.shutdown_census();
        }
        let Some(owner) = &self.owner else {
            return self.shutdown_census();
        };
        let Some(_shutdown_guard) = owner.try_lock_shutdown() else {
            return self.shutdown_census();
        };

        let remaining = timeout.saturating_sub(started_at.elapsed());
        if !self
            .submission_tracker
            .wait_until_submissions_quiescent(remaining)
        {
            return self.shutdown_census();
        }
        let owned_pool = owner.lock_pool().take();
        drop(owned_pool);
        let remaining = timeout.saturating_sub(started_at.elapsed());
        let _ = self.worker_threads.wait_and_join(remaining);
        self.shutdown_census()
    }

    pub(super) fn shutdown_census(&self) -> TaskPoolShutdownCensus {
        let workers = self.worker_threads.census();
        TaskPoolShutdownCensus {
            kind: self.kind(),
            active_submission_count: self.submission_tracker.active_submission_count(),
            expected_worker_count: workers.expected_worker_count,
            exited_worker_count: workers.exited_worker_count,
            joined_worker_count: workers.joined_worker_count,
            termination_signalled: self.termination_signalled.load(Ordering::Acquire),
        }
    }

    fn submission_or_panic(&self) -> TaskPoolSubmission {
        self.try_acquire_submission()
            .unwrap_or_else(|| panic!("{:?} task pool is closing admission", self.kind()))
    }
}

impl Clone for TaskPool {
    fn clone(&self) -> Self {
        Self {
            descriptor: Arc::clone(&self.descriptor),
            parallelism: self.parallelism,
            pool: self.pool.clone(),
            submission_tracker: Arc::clone(&self.submission_tracker),
            termination_signalled: Arc::clone(&self.termination_signalled),
            worker_threads: self.worker_threads.clone(),
            owner: None,
        }
    }
}

impl TaskPoolOwner {
    fn lock_pool(&self) -> MutexGuard<'_, Option<Arc<OwnedRayonPool>>> {
        self.pool
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn try_lock_shutdown(&self) -> Option<MutexGuard<'_, ()>> {
        match self.shutdown_gate.try_lock() {
            Ok(guard) => Some(guard),
            Err(TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) => None,
        }
    }
}

impl OwnedRayonPool {
    fn pool(&self) -> &ThreadPool {
        self.pool
            .as_ref()
            .expect("owned Rayon pool must exist until its final strong handle drops")
    }

    fn try_acquire_submission(pool: Arc<Self>) -> Option<TaskPoolSubmission> {
        if !pool.submission_tracker.try_acquire_submission(false) {
            return None;
        }
        Some(TaskPoolSubmission::new(&pool))
    }

    fn try_acquire_continuation(pool: Arc<Self>) -> Option<TaskPoolSubmission> {
        if !pool.submission_tracker.try_acquire_submission(true) {
            return None;
        }
        Some(TaskPoolSubmission::new(&pool))
    }
}

impl TaskPoolSubmissionTracker {
    fn try_acquire_submission(&self, continuation: bool) -> bool {
        let mut state = self.state.load(Ordering::Acquire);
        loop {
            let active = state & TASK_POOL_ACTIVE_SUBMISSION_MASK;
            let closed = state & TASK_POOL_SUBMISSIONS_CLOSED != 0;
            if active == TASK_POOL_ACTIVE_SUBMISSION_MASK
                || (closed && (!continuation || active == 0))
            {
                return false;
            }
            match self.state.compare_exchange_weak(
                state,
                state + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(observed) => state = observed,
            }
        }
    }

    fn close_admission(&self) {
        self.state
            .fetch_or(TASK_POOL_SUBMISSIONS_CLOSED, Ordering::AcqRel);
    }

    fn wait_until_submissions_quiescent(&self, timeout: Duration) -> bool {
        if self.active_submission_count() == 0 {
            return true;
        }
        let wait = self.lock_quiescent_wait();
        let (_wait, _) = self
            .submissions_quiescent
            .wait_timeout_while(wait, timeout, |_| self.active_submission_count() != 0)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.active_submission_count() == 0
    }

    fn release_submission(&self) {
        let previous = self
            .state
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |state| {
                let active = state & TASK_POOL_ACTIVE_SUBMISSION_MASK;
                active.checked_sub(1).map(|_| state - 1)
            })
            .expect("every task-pool submission must hold an active lease");
        let previous_active = previous & TASK_POOL_ACTIVE_SUBMISSION_MASK;
        if previous_active == 1 {
            let _wait = self.lock_quiescent_wait();
            self.submissions_quiescent.notify_all();
        }
    }

    fn active_submission_count(&self) -> usize {
        self.state.load(Ordering::Acquire) & TASK_POOL_ACTIVE_SUBMISSION_MASK
    }

    fn lock_quiescent_wait(&self) -> MutexGuard<'_, ()> {
        self.quiescent_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl TaskPoolSubmission {
    fn new(pool: &Arc<OwnedRayonPool>) -> Self {
        Self {
            pool: Arc::downgrade(pool),
            tracker: Arc::clone(&pool.submission_tracker),
        }
    }

    pub(super) fn spawn(self, task: impl FnOnce() + Send + 'static) {
        let pool = self.pool_or_panic();
        pool.pool().spawn(move || {
            let _submission = self;
            task();
        });
    }

    fn install<R: Send>(&self, task: impl FnOnce() -> R + Send) -> R {
        self.pool_or_panic().pool().install(task)
    }

    fn in_place_scope<'scope, OP, R>(&self, task: OP) -> R
    where
        OP: FnOnce(&rayon::Scope<'scope>) -> R,
    {
        self.pool_or_panic().pool().in_place_scope(task)
    }

    fn join<A, B, RA, RB>(&self, task_a: A, task_b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.pool_or_panic()
            .pool()
            .install(|| rayon::join(task_a, task_b))
    }

    fn pool_or_panic(&self) -> Arc<OwnedRayonPool> {
        self.pool
            .upgrade()
            .unwrap_or_else(|| panic!("task-pool owner stopped after accepting a submission"))
    }
}

impl Drop for TaskPoolSubmission {
    fn drop(&mut self) {
        self.tracker.release_submission();
    }
}

impl Drop for OwnedRayonPool {
    fn drop(&mut self) {
        let pool = self.pool.take();
        drop(pool);
        self.termination_signalled.store(true, Ordering::Release);
    }
}

pub(super) fn assist_current_thread_once() -> Option<TaskPoolYield> {
    rayon::yield_now().map(|result| match result {
        rayon::Yield::Executed => TaskPoolYield::Executed,
        rayon::Yield::Idle => TaskPoolYield::Idle,
    })
}

impl fmt::Debug for TaskPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskPool")
            .field("kind", &self.descriptor.kind)
            .field("thread_name", &self.descriptor.thread_name)
            .field("parallelism", &self.parallelism)
            .finish()
    }
}

fn default_parallelism() -> usize {
    std::thread::available_parallelism().map_or(1, |value| value.get())
}

#[cfg(test)]
#[path = "pool/tests.rs"]
mod tests;
