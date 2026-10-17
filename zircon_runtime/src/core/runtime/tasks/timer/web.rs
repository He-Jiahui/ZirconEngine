//! Browser timers keep registrations bounded and never block an execution worker.
use super::callback_dispatcher::TaskCallbackDispatcher;
use crate::core::{CoreError, CoreResult};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};
use web_time::{Duration, Instant};

const DEFAULT_TIMER_CAPACITY: usize = 512;
static PROCESS_TIMER: OnceLock<Result<TaskTimer, String>> = OnceLock::new();
static NEXT_TIMER: AtomicUsize = AtomicUsize::new(1);
thread_local! {
    // JavaScript values stay on the browser thread; the shared owner contains only an ID.
    static WAITS: std::cell::RefCell<HashMap<usize, (i32, js_sys::Function)>> = Default::default();
}
pub(crate) struct TaskTimer {
    inner: Arc<Inner>,
}
pub(crate) struct TaskTimerSubscription {
    inner: Weak<Inner>,
    registration: Arc<Registration>,
}
struct Inner {
    id: usize,
    dispatcher: TaskCallbackDispatcher,
    owners: AtomicUsize,
    closed: AtomicBool,
    driving: AtomicBool,
    state: Mutex<State>,
}
struct State {
    capacity: usize,
    next_id: u64,
    deadlines: BTreeMap<Instant, Vec<Arc<Registration>>>,
    index: HashMap<u64, Instant>,
}
struct Registration {
    id: u64,
    interval: Option<Duration>,
    cancelled: AtomicBool,
    delivery_pending: AtomicBool,
    callback: Box<dyn Fn() + Send + Sync + 'static>,
}
impl TaskTimer {
    pub(crate) fn process_default() -> CoreResult<Self> {
        match PROCESS_TIMER
            .get_or_init(|| Self::new(DEFAULT_TIMER_CAPACITY).map_err(|e| e.to_string()))
        {
            Ok(timer) => Ok(timer.clone()),
            Err(error) => Err(CoreError::ThreadSpawn(error.clone())),
        }
    }
    pub(crate) fn new(capacity: usize) -> CoreResult<Self> {
        Self::new_with_callback_dispatcher(capacity, TaskCallbackDispatcher::process_default())
    }
    pub(super) fn new_owned(dispatcher: TaskCallbackDispatcher) -> CoreResult<Self> {
        Self::new_with_callback_dispatcher(DEFAULT_TIMER_CAPACITY, dispatcher)
    }
    pub(super) fn new_with_callback_dispatcher(
        capacity: usize,
        dispatcher: TaskCallbackDispatcher,
    ) -> CoreResult<Self> {
        if web_sys::window().is_none() {
            return Err(CoreError::ThreadSpawn(
                "browser timers require the Window thread".into(),
            ));
        }
        let id = NEXT_TIMER
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| CoreError::ChannelSend("browser timer identity exhausted".into()))?;
        Ok(Self {
            inner: Arc::new(Inner {
                id,
                dispatcher,
                owners: AtomicUsize::new(1),
                closed: AtomicBool::new(false),
                driving: AtomicBool::new(false),
                state: Mutex::new(State {
                    capacity,
                    next_id: 0,
                    deadlines: BTreeMap::new(),
                    index: HashMap::new(),
                }),
            }),
        })
    }
    pub(crate) fn schedule_interval(
        &self,
        interval: Duration,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> CoreResult<TaskTimerSubscription> {
        if interval.is_zero() {
            return Err(CoreError::ChannelSend(
                "runtime timer interval must be non-zero".into(),
            ));
        }
        let deadline = Instant::now()
            .checked_add(interval)
            .ok_or(CoreError::DeadlineOutOfRange)?;
        self.schedule(deadline, Some(interval), callback)
    }
    pub(crate) fn schedule_at(
        &self,
        deadline: Instant,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> CoreResult<TaskTimerSubscription> {
        self.schedule(deadline, None, callback)
    }
    fn schedule(
        &self,
        deadline: Instant,
        interval: Option<Duration>,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> CoreResult<TaskTimerSubscription> {
        let mut state = lock(&self.inner);
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(CoreError::ChannelSend(
                "runtime timer is shutting down".into(),
            ));
        }
        if state.index.len() >= state.capacity {
            return Err(CoreError::ChannelSend(
                "runtime timer registration capacity full".into(),
            ));
        }
        let id = state.next_id;
        state.next_id = id
            .checked_add(1)
            .ok_or_else(|| CoreError::ChannelSend("runtime timer id space exhausted".into()))?;
        let registration = Arc::new(Registration {
            id,
            interval,
            cancelled: AtomicBool::new(false),
            delivery_pending: AtomicBool::new(false),
            callback: Box::new(callback),
        });
        state
            .deadlines
            .entry(deadline)
            .or_default()
            .push(registration.clone());
        state.index.insert(id, deadline);
        drop(state);
        wake(&self.inner);
        Ok(TaskTimerSubscription {
            inner: Arc::downgrade(&self.inner),
            registration,
        })
    }
    pub(super) fn close_admission(&self) {
        let state = lock(&self.inner);
        self.inner.closed.store(true, Ordering::Release);
        drop(state);
        cancel_wait(self.inner.id);
    }
    pub(super) fn shutdown_until(&self, _deadline: Instant) -> bool {
        self.close_admission();
        clear(&self.inner);
        true // This backend has no physical worker to join.
    }
    pub(super) fn is_joined(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }
}
impl Clone for TaskTimer {
    fn clone(&self) -> Self {
        self.inner.owners.fetch_add(1, Ordering::AcqRel);
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl Drop for TaskTimer {
    fn drop(&mut self) {
        if self.inner.owners.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.close_admission();
            clear(&self.inner);
        }
    }
}
impl fmt::Debug for TaskTimer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserTaskTimer").finish_non_exhaustive()
    }
}
impl TaskTimerSubscription {
    pub(crate) fn cancel(&self) {
        if self.registration.cancelled.swap(true, Ordering::AcqRel) {
            return;
        }
        let Some(inner) = self.inner.upgrade() else {
            return;
        };
        let mut state = lock(&inner);
        if let Some(deadline) = state.index.remove(&self.registration.id) {
            if let Some(bucket) = state.deadlines.get_mut(&deadline) {
                bucket.retain(|r| r.id != self.registration.id);
                if bucket.is_empty() {
                    state.deadlines.remove(&deadline);
                }
            }
        }
        drop(state);
        wake(&inner);
    }
}
impl Drop for TaskTimerSubscription {
    fn drop(&mut self) {
        self.cancel();
    }
}
fn clear(inner: &Inner) {
    let pending = {
        let mut state = lock(inner);
        state.index.clear();
        std::mem::take(&mut state.deadlines)
    };
    drop(pending); // Release user captures outside the state lock.
}
fn lock(inner: &Inner) -> MutexGuard<'_, State> {
    inner.state.lock().unwrap_or_else(|e| e.into_inner())
}
fn cancel_wait(id: usize) {
    let pending = WAITS.with(|waits| waits.borrow_mut().remove(&id));
    if let Some((handle, resolve)) = pending {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(handle);
        }
        let _ = resolve.call0(&wasm_bindgen::JsValue::UNDEFINED);
    }
}
fn wake(inner: &Arc<Inner>) {
    cancel_wait(inner.id);
    if inner.closed.load(Ordering::Acquire) || inner.driving.swap(true, Ordering::AcqRel) {
        return;
    }
    let weak = Arc::downgrade(inner);
    wasm_bindgen_futures::spawn_local(async move {
        drive(weak).await;
    });
}
fn take_due(inner: &Inner) -> Vec<Arc<Registration>> {
    let mut state = lock(inner);
    let Some((&deadline, _)) = state.deadlines.first_key_value() else {
        return Vec::new();
    };
    let now = Instant::now();
    if now < deadline {
        return Vec::new();
    }
    let mut due = state
        .deadlines
        .remove(&deadline)
        .expect("selected deadline exists");
    due.retain(|registration| {
        if state.index.remove(&registration.id).is_none()
            || registration.cancelled.load(Ordering::Acquire)
        {
            return false;
        }
        if let Some(interval) = registration.interval {
            let Some(next) = now.checked_add(interval) else {
                registration.cancelled.store(true, Ordering::Release);
                return false;
            };
            state
                .deadlines
                .entry(next)
                .or_default()
                .push(registration.clone());
            state.index.insert(registration.id, next);
        }
        !registration.delivery_pending.swap(true, Ordering::AcqRel)
    });
    due
}
async fn drive(weak: Weak<Inner>) {
    loop {
        let Some(inner) = weak.upgrade() else {
            return;
        };
        if inner.closed.load(Ordering::Acquire) {
            inner.driving.store(false, Ordering::Release);
            return;
        }
        for registration in take_due(&inner) {
            let delivery_owner = weak.clone();
            inner.dispatcher.dispatch_one(Box::new(move || {
                struct Delivery(Arc<Registration>);
                impl Drop for Delivery {
                    fn drop(&mut self) {
                        self.0.delivery_pending.store(false, Ordering::Release);
                    }
                }
                let _delivery = Delivery(registration.clone());
                if delivery_owner
                    .upgrade()
                    .is_some_and(|inner| !inner.closed.load(Ordering::Acquire))
                    && !registration.cancelled.load(Ordering::Acquire)
                {
                    (registration.callback)();
                }
            }));
        }
        let deadline = lock(&inner)
            .deadlines
            .first_key_value()
            .map(|(deadline, _)| *deadline);
        let Some(deadline) = deadline else {
            inner.driving.store(false, Ordering::Release);
            return;
        };
        let id = inner.id;
        let delay = deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(i32::MAX as u128) as i32;
        let promise = js_sys::Promise::new(&mut |resolve, reject| match web_sys::window()
            .expect("browser Window thread")
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay)
        {
            Ok(handle) => {
                WAITS.with(|waits| {
                    assert!(waits.borrow_mut().insert(id, (handle, resolve)).is_none());
                });
            }
            Err(error) => {
                let _ = reject.call1(&wasm_bindgen::JsValue::UNDEFINED, &error);
            }
        });
        drop(inner); // No timer or user capture is retained across the browser wait.
        let result = wasm_bindgen_futures::JsFuture::from(promise).await;
        WAITS.with(|waits| {
            waits.borrow_mut().remove(&id);
        });
        if result.is_err() {
            if let Some(inner) = weak.upgrade() {
                inner.closed.store(true, Ordering::Release);
                inner.driving.store(false, Ordering::Release);
                clear(&inner);
            }
            return;
        }
    }
}
