//! Process-owned delayed callbacks for runtime lifecycle maintenance.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::core::{CoreError, CoreResult};

use super::spawn_named_thread;

const PROCESS_TIMER_CAPACITY: usize = 512;
const PROCESS_TIMER_THREAD_NAME: &str = "zircon-runtime-timer";

static PROCESS_TIMER: OnceLock<Result<TaskTimer, String>> = OnceLock::new();

/// A process-wide timer service for small lifecycle callbacks.
pub(crate) struct TaskTimer {
    inner: Arc<TaskTimerInner>,
    worker: Arc<TaskTimerWorker>,
}

/// Cancels one recurring callback when dropped.
pub(crate) struct TaskTimerSubscription {
    timer: Weak<TaskTimerInner>,
    registration: Arc<TimerRegistration>,
}

struct TaskTimerInner {
    state: Mutex<TaskTimerState>,
    changed: Condvar,
    owners: AtomicUsize,
    closing: AtomicBool,
}

struct TaskTimerWorker {
    join_handle: Mutex<Option<JoinHandle<()>>>,
}

struct TaskTimerState {
    next_id: u64,
    capacity: usize,
    deadlines: BTreeMap<Instant, Vec<Arc<TimerRegistration>>>,
    scheduled_deadlines: HashMap<u64, Instant>,
}

struct TimerRegistration {
    id: u64,
    schedule: TimerSchedule,
    cancelled: AtomicBool,
    callback: Box<dyn Fn() + Send + Sync + 'static>,
}

#[derive(Clone, Copy)]
enum TimerSchedule {
    Once,
    Interval(Duration),
}

impl TaskTimer {
    /// Returns the single control-plane timer shared by all runtime instances.
    pub(crate) fn process_default() -> CoreResult<Self> {
        match PROCESS_TIMER
            .get_or_init(|| Self::new(PROCESS_TIMER_CAPACITY).map_err(|error| error.to_string()))
        {
            Ok(timer) => Ok(timer.clone()),
            Err(error) => Err(CoreError::ThreadSpawn(error.clone())),
        }
    }

    /// Registers a recurring callback without assigning a permanently blocked pool worker.
    pub(crate) fn schedule_interval(
        &self,
        interval: Duration,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> CoreResult<TaskTimerSubscription> {
        if interval.is_zero() {
            return Err(CoreError::ChannelSend(
                "runtime timer interval must be non-zero".to_string(),
            ));
        }
        let now = Instant::now();
        let deadline = now.checked_add(interval).unwrap_or(now);
        self.schedule(TimerSchedule::Interval(interval), deadline, callback)
    }

    /// Schedules one lifecycle callback at its exact deadline.
    pub(crate) fn schedule_at(
        &self,
        deadline: Instant,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> CoreResult<TaskTimerSubscription> {
        self.schedule(TimerSchedule::Once, deadline, callback)
    }

    fn schedule(
        &self,
        schedule: TimerSchedule,
        deadline: Instant,
        callback: impl Fn() + Send + Sync + 'static,
    ) -> CoreResult<TaskTimerSubscription> {
        if self.inner.closing.load(Ordering::Acquire) {
            return Err(CoreError::ChannelSend(
                "runtime timer is shutting down".to_string(),
            ));
        }
        let mut state = lock_timer_state(&self.inner);
        if self.inner.closing.load(Ordering::Acquire) {
            return Err(CoreError::ChannelSend(
                "runtime timer is shutting down".to_string(),
            ));
        }
        if state.scheduled_deadlines.len() >= state.capacity {
            return Err(CoreError::ChannelSend(
                "runtime timer registration capacity full".to_string(),
            ));
        }
        let id = state.next_id;
        state.next_id = state.next_id.checked_add(1).ok_or_else(|| {
            CoreError::ChannelSend("runtime timer id space exhausted".to_string())
        })?;
        let registration = Arc::new(TimerRegistration {
            id,
            schedule,
            cancelled: AtomicBool::new(false),
            callback: Box::new(callback),
        });
        state
            .deadlines
            .entry(deadline)
            .or_default()
            .push(Arc::clone(&registration));
        state.scheduled_deadlines.insert(id, deadline);
        drop(state);
        self.inner.changed.notify_one();

        Ok(TaskTimerSubscription {
            timer: Arc::downgrade(&self.inner),
            registration,
        })
    }

    /// Creates an explicitly bounded timer for a contained runtime owner.
    pub(crate) fn new(capacity: usize) -> CoreResult<Self> {
        let inner = Arc::new(TaskTimerInner {
            state: Mutex::new(TaskTimerState {
                next_id: 0,
                capacity,
                deadlines: BTreeMap::new(),
                scheduled_deadlines: HashMap::new(),
            }),
            changed: Condvar::new(),
            owners: AtomicUsize::new(1),
            closing: AtomicBool::new(false),
        });
        let worker = Arc::new(TaskTimerWorker {
            join_handle: Mutex::new(None),
        });
        let timer = Self { inner, worker };
        let runner = Arc::downgrade(&timer.inner);
        let handle = spawn_named_thread(PROCESS_TIMER_THREAD_NAME, move || run_timer(runner))?;
        *lock_timer_worker(&timer.worker) = Some(handle);
        Ok(timer)
    }
}

impl Clone for TaskTimer {
    fn clone(&self) -> Self {
        self.inner.owners.fetch_add(1, Ordering::AcqRel);
        Self {
            inner: Arc::clone(&self.inner),
            worker: Arc::clone(&self.worker),
        }
    }
}

impl Drop for TaskTimer {
    fn drop(&mut self) {
        if self.inner.owners.fetch_sub(1, Ordering::AcqRel) != 1 {
            return;
        }

        self.inner.closing.store(true, Ordering::Release);
        self.inner.changed.notify_all();
        let handle = {
            let mut handle = lock_timer_worker(&self.worker);
            if handle
                .as_ref()
                .is_some_and(|worker| worker.thread().id() == thread::current().id())
            {
                None
            } else {
                handle.take()
            }
        };
        if let Some(handle) = handle {
            let _ = handle.join();
        }
    }
}

impl fmt::Debug for TaskTimer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("TaskTimer").finish_non_exhaustive()
    }
}

impl TaskTimerSubscription {
    pub(crate) fn cancel(&self) {
        if self.registration.cancelled.swap(true, Ordering::AcqRel) {
            return;
        }
        let Some(timer) = self.timer.upgrade() else {
            return;
        };
        let mut state = lock_timer_state(&timer);
        let Some(deadline) = state.scheduled_deadlines.remove(&self.registration.id) else {
            return;
        };
        let mut remove_deadline = false;
        if let Some(registrations) = state.deadlines.get_mut(&deadline) {
            registrations.retain(|registration| registration.id != self.registration.id);
            remove_deadline = registrations.is_empty();
        }
        if remove_deadline {
            state.deadlines.remove(&deadline);
        }
        drop(state);
        timer.changed.notify_one();
    }
}

impl Drop for TaskTimerSubscription {
    fn drop(&mut self) {
        self.cancel();
    }
}

impl fmt::Debug for TaskTimerSubscription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TaskTimerSubscription")
            .field("id", &self.registration.id)
            .finish_non_exhaustive()
    }
}

fn run_timer(timer: Weak<TaskTimerInner>) {
    while let Some(timer) = timer.upgrade() {
        let Some(callbacks) = next_callbacks(&timer) else {
            return;
        };
        drop(timer);
        for registration in callbacks {
            if !registration.cancelled.load(Ordering::Acquire) {
                let _ = catch_unwind(AssertUnwindSafe(|| (registration.callback)()));
            }
        }
    }
}

fn next_callbacks(timer: &TaskTimerInner) -> Option<Vec<Arc<TimerRegistration>>> {
    let mut state = lock_timer_state(timer);
    loop {
        if timer.closing.load(Ordering::Acquire) {
            return None;
        }
        let Some((&deadline, _)) = state.deadlines.first_key_value() else {
            state = timer
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            continue;
        };
        let now = Instant::now();
        if now < deadline {
            let wait = deadline.saturating_duration_since(now);
            let (next_state, _) = timer
                .changed
                .wait_timeout(state, wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            continue;
        }

        let registrations = state
            .deadlines
            .remove(&deadline)
            .expect("timer deadline exists while selected");
        let next_deadline = Instant::now();
        let mut callbacks = Vec::with_capacity(registrations.len());
        for registration in registrations {
            if state.scheduled_deadlines.remove(&registration.id).is_none()
                || registration.cancelled.load(Ordering::Acquire)
            {
                continue;
            }
            if let TimerSchedule::Interval(interval) = registration.schedule {
                let deadline = next_deadline.checked_add(interval).unwrap_or(next_deadline);
                state
                    .deadlines
                    .entry(deadline)
                    .or_default()
                    .push(Arc::clone(&registration));
                state.scheduled_deadlines.insert(registration.id, deadline);
            }
            callbacks.push(registration);
        }
        return Some(callbacks);
    }
}

fn lock_timer_state(timer: &TaskTimerInner) -> MutexGuard<'_, TaskTimerState> {
    timer
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_timer_worker(worker: &TaskTimerWorker) -> MutexGuard<'_, Option<JoinHandle<()>>> {
    worker
        .join_handle
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::TaskTimer;

    #[test]
    fn task_timer_rejects_zero_interval() {
        let timer = TaskTimer::process_default().expect("process timer should start");
        let error = timer
            .schedule_interval(Duration::ZERO, || {})
            .expect_err("zero interval must be rejected");
        assert!(error.to_string().contains("must be non-zero"));
    }

    #[test]
    fn dropping_a_subscription_releases_its_bounded_timer_slot() {
        let timer = TaskTimer::new(1).expect("test timer should start");
        let subscription = timer
            .schedule_at(Instant::now() + Duration::from_secs(1), || {})
            .expect("first registration should fit the timer capacity");

        let error = timer
            .schedule_at(Instant::now() + Duration::from_secs(1), || {})
            .expect_err("second registration must observe the hard timer capacity");
        assert!(error.to_string().contains("registration capacity full"));

        drop(subscription);
        let replacement = timer
            .schedule_at(Instant::now() + Duration::from_secs(1), || {})
            .expect("dropping a subscription must release its timer slot");
        drop(replacement);
    }

    #[test]
    fn dropping_the_last_explicit_timer_owner_stops_its_worker() {
        let timer = TaskTimer::new(1).expect("test timer should start");
        let inner = Arc::downgrade(&timer.inner);

        drop(timer);

        assert!(
            inner.upgrade().is_none(),
            "the explicit timer worker must not retain its state after its final owner drops"
        );
    }

    #[test]
    fn dropped_subscription_stops_recurring_callbacks() {
        let timer = TaskTimer::process_default().expect("process timer should start");
        let (first_callback_tx, first_callback_rx) = mpsc::sync_channel(1);
        let callback_count = Arc::new(AtomicUsize::new(0));
        let callback_count_for_task = Arc::clone(&callback_count);
        let subscription = timer
            .schedule_interval(Duration::from_millis(1), move || {
                callback_count_for_task.fetch_add(1, Ordering::AcqRel);
                let _ = first_callback_tx.try_send(());
            })
            .expect("timer should accept one recurring callback");

        first_callback_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("recurring callback should run");
        drop(subscription);
        std::thread::sleep(Duration::from_millis(10));
        let settled_count = callback_count.load(Ordering::Acquire);
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(callback_count.load(Ordering::Acquire), settled_count);
    }

    #[test]
    fn panicking_callback_does_not_stop_later_timer_callbacks() {
        let timer = TaskTimer::process_default().expect("process timer should start");
        let panic_started = Arc::new(AtomicBool::new(false));
        let panic_started_for_task = Arc::clone(&panic_started);
        let panicking_subscription = timer
            .schedule_interval(Duration::from_millis(1), move || {
                if !panic_started_for_task.swap(true, Ordering::AcqRel) {
                    panic!("timer callback panic");
                }
            })
            .expect("timer should accept a panicking callback");

        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while !panic_started.load(Ordering::Acquire) {
            assert!(
                std::time::Instant::now() < deadline,
                "panicking callback should run before the healthy subscription"
            );
            std::thread::sleep(Duration::from_millis(1));
        }

        let (healthy_callback_tx, healthy_callback_rx) = mpsc::sync_channel(1);
        let healthy_subscription = timer
            .schedule_interval(Duration::from_millis(1), move || {
                let _ = healthy_callback_tx.try_send(());
            })
            .expect("timer should accept a healthy callback after a panic");
        healthy_callback_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("timer should continue after a callback panic");
        drop(healthy_subscription);
        drop(panicking_subscription);
    }
}
