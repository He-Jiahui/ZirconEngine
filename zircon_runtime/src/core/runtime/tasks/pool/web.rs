//! Browser execution domain: bounded cooperative work, with no worker threads.
use super::{TaskPoolDescriptor, TaskPoolKind};
use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use web_time::{Duration, Instant};

const TASK_CAPACITY: usize = 4096;
const TASKS_PER_TURN: usize = 64;
type Task = Box<dyn FnOnce() + Send + 'static>;
thread_local! {
    static POOLS: std::cell::RefCell<Vec<Weak<Inner>>> = const { std::cell::RefCell::new(Vec::new()) };
    static CURRENT: Cell<usize> = const { Cell::new(0) };
}
struct State {
    queue: VecDeque<Task>,
    active: usize,
    closed: bool,
}
struct Inner {
    state: Mutex<State>,
    scheduled: AtomicBool,
}
struct Owner(Arc<Inner>);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TaskPoolYield {
    Executed,
    Idle,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskPoolBuildError {
    kind: TaskPoolKind,
    message: String,
}
impl fmt::Display for TaskPoolBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to build {:?} browser task pool: {}",
            self.kind, self.message
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
/// Clones retain execution access, while only the constructor owns admission.
pub struct TaskPool {
    descriptor: Arc<TaskPoolDescriptor>,
    inner: Weak<Inner>,
    _owner: Option<Arc<Owner>>,
}
pub(in crate::core::runtime) struct TaskPoolSubmission {
    inner: Arc<Inner>,
}
impl TaskPool {
    pub fn new(descriptor: TaskPoolDescriptor) -> Self {
        Self::try_new(descriptor).unwrap_or_else(|error| panic!("{error}"))
    }
    pub fn try_new(descriptor: TaskPoolDescriptor) -> Result<Self, TaskPoolBuildError> {
        #[cfg(target_arch = "wasm32")]
        if web_sys::window().is_none() {
            return Err(TaskPoolBuildError {
                kind: descriptor.kind,
                message: "the browser runtime must be started on the Window thread".into(),
            });
        }
        let inner = Arc::new(Inner {
            state: Mutex::new(State {
                queue: VecDeque::new(),
                active: 0,
                closed: false,
            }),
            scheduled: AtomicBool::new(false),
        });
        POOLS.with(|pools| {
            let mut pools = pools.borrow_mut();
            pools.retain(|pool| pool.strong_count() > 0);
            pools.push(Arc::downgrade(&inner));
        });
        Ok(Self {
            descriptor: Arc::new(descriptor),
            inner: Arc::downgrade(&inner),
            _owner: Some(Arc::new(Owner(inner))),
        })
    }
    pub fn kind(&self) -> TaskPoolKind {
        self.descriptor.kind
    }
    pub fn descriptor(&self) -> &TaskPoolDescriptor {
        &self.descriptor
    }
    /// Logical execution width; this backend creates zero physical workers.
    pub fn parallelism(&self) -> usize {
        1
    }
    pub fn shares_execution_owner_with(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.inner, &other.inner)
    }
    pub(crate) fn is_current_worker(&self) -> bool {
        CURRENT.with(|current| current.get() == self.inner.as_ptr() as usize)
    }
    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        self.submission().spawn(task);
    }
    pub(crate) fn try_spawn<F: FnOnce() + Send + 'static>(&self, task: F) -> Result<(), F> {
        let Some(submission) = self.try_acquire_submission() else {
            return Err(task);
        };
        submission.spawn(task);
        Ok(())
    }
    pub fn install<R: Send>(&self, task: impl FnOnce() -> R + Send) -> R {
        let _lease = self.submission();
        task()
    }
    pub(crate) fn in_place_scope<'scope, OP, R>(&self, task: OP) -> R
    where
        OP: FnOnce(&WebScope<'scope>) -> R,
    {
        let _lease = self.submission();
        task(&WebScope(PhantomData))
    }
    pub fn join<A, B, RA, RB>(&self, a: A, b: B) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        self.install(|| (a(), b()))
    }
    pub(super) fn close_admission(&self) {
        if let Some(inner) = self.inner.upgrade() {
            inner.state.lock().unwrap_or_else(|e| e.into_inner()).closed = true;
        }
    }
    pub(super) fn try_acquire_submission(&self) -> Option<TaskPoolSubmission> {
        self.acquire(false)
    }
    pub(super) fn try_acquire_continuation(&self) -> Option<TaskPoolSubmission> {
        self.acquire(true)
    }
    fn acquire(&self, continuation: bool) -> Option<TaskPoolSubmission> {
        let inner = self.inner.upgrade()?;
        let mut state = inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.active >= TASK_CAPACITY || (state.closed && (!continuation || state.active == 0)) {
            return None;
        }
        state.active += 1;
        drop(state);
        Some(TaskPoolSubmission { inner })
    }
    fn submission(&self) -> TaskPoolSubmission {
        self.try_acquire_submission().expect(
            "browser task pool is full or shutting down; bounded callers must use admission",
        )
    }
    pub(crate) fn shutdown_until(&self, deadline: Instant) -> bool {
        self.close_and_join(deadline.saturating_duration_since(Instant::now()))
            .termination_signalled
    }
    pub(super) fn close_and_join(&self, timeout: Duration) -> TaskPoolShutdownCensus {
        self.close_admission();
        if let Some(inner) = self.inner.upgrade() {
            let started = Instant::now();
            for _ in 0..TASKS_PER_TURN {
                if started.elapsed() >= timeout || !run_one(&inner) {
                    break;
                }
            }
        }
        self.shutdown_census()
    }
    pub(super) fn shutdown_census(&self) -> TaskPoolShutdownCensus {
        let (active, closed) = self.inner.upgrade().map_or((0, true), |inner| {
            let state = inner.state.lock().unwrap_or_else(|e| e.into_inner());
            (state.active, state.closed)
        });
        TaskPoolShutdownCensus {
            kind: self.kind(),
            active_submission_count: active,
            expected_worker_count: 0,
            exited_worker_count: 0,
            joined_worker_count: 0,
            termination_signalled: closed && active == 0,
        }
    }
}
impl Clone for TaskPool {
    fn clone(&self) -> Self {
        Self {
            descriptor: self.descriptor.clone(),
            inner: self.inner.clone(),
            _owner: None,
        }
    }
}
impl fmt::Debug for TaskPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserTaskPool")
            .field("kind", &self.kind())
            .field("parallelism", &1)
            .finish()
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.0
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .closed = true;
    }
}
impl TaskPoolSubmission {
    pub(super) fn spawn(self, task: impl FnOnce() + Send + 'static) {
        let inner = self.inner.clone();
        inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .queue
            .push_back(Box::new(move || {
                let _lease = self;
                task();
            }));
        schedule(&inner);
    }
}
impl Drop for TaskPoolSubmission {
    fn drop(&mut self) {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state.active = state
            .active
            .checked_sub(1)
            .expect("submission lease accounting");
    }
}
/// Borrowed parallel scopes execute immediately and retain no borrowed closures.
pub(crate) struct WebScope<'scope>(PhantomData<&'scope ()>);
impl<'scope> WebScope<'scope> {
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce(&Self) + Send + 'scope,
    {
        task(self);
    }
}
fn run_one(inner: &Arc<Inner>) -> bool {
    let task = inner
        .state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .queue
        .pop_front();
    let Some(task) = task else {
        return false;
    };
    struct Restore(usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            CURRENT.with(|current| current.set(self.0));
        }
    }
    let _restore = Restore(CURRENT.with(|current| current.replace(Arc::as_ptr(inner) as usize)));
    task();
    true
}
pub(super) fn assist_current_thread_once() -> Option<TaskPoolYield> {
    let inner = POOLS.with(|pools| {
        pools
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .find(|inner| {
                !inner
                    .state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .queue
                    .is_empty()
            })
    });
    Some(if inner.is_some_and(|inner| run_one(&inner)) {
        TaskPoolYield::Executed
    } else {
        TaskPoolYield::Idle
    })
}
#[cfg(not(target_arch = "wasm32"))]
fn schedule(_inner: &Arc<Inner>) {}
#[cfg(target_arch = "wasm32")]
fn schedule(inner: &Arc<Inner>) {
    if inner.scheduled.swap(true, Ordering::AcqRel) {
        return;
    }
    let inner = inner.clone();
    wasm_bindgen_futures::spawn_local(async move {
        // A timer turn yields to browser input and paint; a self-reposting
        // microtask loop would starve them under sustained publication.
        let promise = js_sys::Promise::new(&mut |resolve, reject| match web_sys::window()
            .expect("browser Window thread")
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
        {
            Ok(_) => {}
            Err(error) => {
                let _ = reject.call1(&wasm_bindgen::JsValue::UNDEFINED, &error);
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        for _ in 0..TASKS_PER_TURN {
            if !run_one(&inner) {
                break;
            }
        }
        inner.scheduled.store(false, Ordering::Release);
        let pending = !inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .queue
            .is_empty();
        if pending {
            schedule(&inner);
        }
    });
}

#[cfg(test)]
#[path = "web/tests.rs"]
mod tests;
